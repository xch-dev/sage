use chia::protocol::{Bytes32, CoinSpend};
use chia_wallet_sdk::{Conditions, Did, HashedPtr, Launcher, SpendContext, StandardLayer};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    pub async fn create_did(
        &self,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Did<()>), WalletError> {
        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let synthetic_key = self.db.synthetic_key(coins[0].puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);
        let (mut conditions, did) =
            Launcher::new(coins[0].coin_id(), 1).create_simple_did(&mut ctx, &p2)?;

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), did))
    }

    pub async fn transfer_dids(
        &self,
        did_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if did_ids.is_empty() {
            return Err(WalletError::EmptyBulkTransfer);
        }

        let mut dids = Vec::new();

        for did_id in did_ids {
            let Some(did) = self.db.spendable_did(did_id).await? else {
                return Err(WalletError::MissingDid(did_id));
            };

            dids.push(did);
        }

        let coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did_coin_ids = dids
            .iter()
            .map(|did| did.coin.coin_id())
            .collect::<Vec<_>>();

        for (i, did) in dids.into_iter().enumerate() {
            let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
            let did = did.with_metadata(HashedPtr::from_ptr(&ctx.allocator, did_metadata_ptr));

            let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = if did_coin_ids.len() == 1 {
                Conditions::new()
            } else {
                Conditions::new().assert_concurrent_spend(
                    did_coin_ids[if i == 0 {
                        did_coin_ids.len() - 1
                    } else {
                        i - 1
                    }],
                )
            };

            let _did = did.transfer(&mut ctx, &p2, puzzle_hash, conditions)?;
        }

        if fee > 0 {
            let mut conditions = Conditions::new()
                .assert_concurrent_spend(did_coin_ids[0])
                .reserve_fee(fee);

            if change > 0 {
                conditions = conditions.create_coin(p2_puzzle_hash, change, None);
            }

            self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        }

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use crate::TestWallet;

    use test_log::test;

    #[test(tokio::test)]
    async fn test_create_did() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1).await?;

        let (coin_spends, did) = test.wallet.create_did(0, false, true).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_dids(vec![did.info.launcher_id], test.puzzle_hash, 0, false, true)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        assert_ne!(
            test.wallet.db.spendable_did(did.info.launcher_id).await?,
            None
        );

        Ok(())
    }
}
