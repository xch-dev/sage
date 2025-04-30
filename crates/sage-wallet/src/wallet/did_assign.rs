use chia::protocol::{Bytes32, CoinSpend};
use chia_wallet_sdk::{
    driver::{DidOwner, HashedPtr, SpendContext, StandardLayer},
    types::Conditions,
};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    pub async fn assign_nfts(
        &self,
        nft_ids: Vec<Bytes32>,
        did_id: Option<Bytes32>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if nft_ids.is_empty() {
            return Err(WalletError::EmptyBulkTransfer);
        }

        let mut nfts = Vec::new();

        for nft_id in nft_ids {
            let Some(nft) = self.db.spendable_nft(nft_id).await? else {
                return Err(WalletError::MissingNft(nft_id));
            };

            nfts.push(nft);
        }

        let did = if let Some(did_id) = did_id {
            let did = self
                .db
                .spendable_did(did_id)
                .await?
                .ok_or(WalletError::MissingDid(did_id))?;

            Some(did)
        } else {
            None
        };

        let coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did = if let Some(did) = did {
            let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
            Some(did.with_metadata(HashedPtr::from_ptr(&ctx, did_metadata_ptr)))
        } else {
            None
        };

        let nft_coin_ids = nfts
            .iter()
            .map(|nft| nft.coin.coin_id())
            .collect::<Vec<_>>();

        let mut did_conditions = Conditions::new();

        for (i, nft) in nfts.into_iter().enumerate() {
            let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx, nft_metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = if nft_coin_ids.len() == 1 {
                Conditions::new()
            } else {
                Conditions::new().assert_concurrent_spend(
                    nft_coin_ids[if i == 0 {
                        nft_coin_ids.len() - 1
                    } else {
                        i - 1
                    }],
                )
            };

            let (parent_conditions, _nft) = nft.transfer_to_did(
                &mut ctx,
                &p2,
                nft.info.p2_puzzle_hash,
                did.as_ref().map(|did| DidOwner::from_did_info(&did.info)),
                conditions,
            )?;

            did_conditions = did_conditions.extend(parent_conditions);
        }

        let did_coin_id = did.as_ref().map(|did| did.coin.coin_id());

        if let Some(did) = did {
            let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);
            let _did = did.update(&mut ctx, &p2, did_conditions)?;
        }

        if fee > 0 {
            let mut conditions = Conditions::new()
                .assert_concurrent_spend(nft_coin_ids[0])
                .reserve_fee(fee);

            if change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change, None);
            }

            if let Some(did_coin_id) = did_coin_id {
                conditions = conditions.assert_concurrent_spend(did_coin_id);
            }

            self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        }

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use chia::puzzles::nft::NftMetadata;
    use test_log::test;

    use crate::{TestWallet, WalletNftMint};

    use super::*;

    #[test(tokio::test)]
    async fn test_assign_nft() -> anyhow::Result<()> {
        let mut test = TestWallet::new(2).await?;

        let (coin_spends, did) = test.wallet.create_did(0, false, true).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let nft = nfts.remove(0);

        let coin_spends = test
            .wallet
            .assign_nfts(vec![nft.info.launcher_id], None, 0, false, true)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let coin_spends = test
            .wallet
            .assign_nfts(
                vec![nft.info.launcher_id],
                Some(did.info.launcher_id),
                0,
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        Ok(())
    }
}
