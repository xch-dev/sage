use std::mem;

use chia::{
    bls::PublicKey,
    protocol::{Bytes, Bytes32, CoinSpend},
};
use chia_wallet_sdk::{
    driver::{Cat, SpendContext},
    types::Conditions,
};

use crate::WalletError;

use super::{memos::calculate_memos, Wallet};

impl Wallet {
    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
        multi_issuance_key: Option<PublicKey>,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let total_amount = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let hint = ctx.hint(p2_puzzle_hash)?;

        let eve_conditions = Conditions::new().create_coin(p2_puzzle_hash, amount, Some(hint));

        let (mut conditions, eve) = match multi_issuance_key {
            Some(pk) => {
                Cat::multi_issuance_eve(&mut ctx, coins[0].coin_id(), pk, amount, eve_conditions)?
            }
            None => Cat::single_issuance_eve(&mut ctx, coins[0].coin_id(), amount, eve_conditions)?,
        };

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), eve.asset_id))
    }

    /// Sends the given amount of CAT to the given puzzle hash.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_cat(
        &self,
        asset_id: Bytes32,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };

        let combined_amount = amounts.iter().map(|(_, amount)| amount).sum::<u64>();

        let cats = self
            .select_cat_coins(asset_id, combined_amount as u128)
            .await?;
        let cat_selected: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();
        let cat_change: u64 = (cat_selected - combined_amount as u128)
            .try_into()
            .expect("change amount overflow");

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut conditions = if fee_coins.is_empty() {
            Conditions::new()
        } else {
            Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id())
        };

        if fee > 0 {
            let fee_selected: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
            let fee_change: u64 = (fee_selected - fee as u128)
                .try_into()
                .expect("fee change overflow");

            conditions = conditions.reserve_fee(fee);

            if fee_change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, fee_change, None);
            }
        }

        let mut ctx = SpendContext::new();

        if !fee_coins.is_empty() {
            self.spend_p2_coins(&mut ctx, fee_coins, mem::take(&mut conditions))
                .await?;
        }

        for (puzzle_hash, amount) in amounts {
            conditions = conditions.create_coin(
                puzzle_hash,
                amount,
                calculate_memos(&mut ctx, puzzle_hash, include_hint, memos.clone())?,
            );
        }

        let change_hint = ctx.hint(change_puzzle_hash)?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i != 0 {
                    return (cat, Conditions::new());
                }

                let mut conditions = mem::take(&mut conditions);

                if cat_change > 0 {
                    conditions =
                        conditions.create_coin(change_puzzle_hash, cat_change, Some(change_hint));
                }

                (cat, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::TestWallet;

    #[test(tokio::test)]
    async fn test_send_cat() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1500).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None, false, true).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 500);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_cat(
                asset_id,
                vec![(test.puzzle_hash, 750)],
                0,
                true,
                None,
                false,
                true,
            )
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 2);

        let coin_spends = test
            .wallet
            .send_cat(
                asset_id,
                vec![(test.puzzle_hash, 1000)],
                500,
                true,
                None,
                false,
                true,
            )
            .await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 0);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 0);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        Ok(())
    }
}
