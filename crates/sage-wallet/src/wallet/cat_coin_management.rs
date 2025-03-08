use chia::protocol::CoinSpend;
use chia_wallet_sdk::{
    driver::{Cat, SpendContext},
    types::Conditions,
};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Combines multiple CAT coins into a single coin, with the given fee subtracted from the output.
    pub async fn combine_cat(
        &self,
        cats: Vec<Cat>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };
        let cat_total: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        if !fee_coins.is_empty() {
            let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();

            let fee_change: u64 = (fee_total - fee as u128)
                .try_into()
                .expect("change amount overflow");

            let mut fee_conditions =
                Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

            if fee > 0 {
                fee_conditions = fee_conditions.reserve_fee(fee);
            }

            if fee_change > 0 {
                fee_conditions = fee_conditions.create_coin(p2_puzzle_hash, fee_change, None);
            }

            self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
                .await?;
        }

        let hint = ctx.hint(p2_puzzle_hash)?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i == 0 {
                    (
                        cat,
                        Conditions::new().create_coin(
                            p2_puzzle_hash,
                            cat_total.try_into().expect("output amount overflow"),
                            Some(hint),
                        ),
                    )
                } else {
                    (cat, Conditions::new())
                }
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    /// Splits the given CAT coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_cat(
        &self,
        cats: Vec<Cat>,
        output_count: usize,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };
        let cat_total: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();

        let mut remaining_count = output_count;
        let mut remaining_amount = cat_total;

        let max_individual_amount: u64 = remaining_amount
            .div_ceil(output_count as u128)
            .try_into()
            .expect("output amount overflow");

        let derivations_needed: u32 = output_count
            .div_ceil(cats.len())
            .try_into()
            .expect("derivation count overflow");

        let derivations = self
            .p2_puzzle_hashes(derivations_needed, hardened, reuse)
            .await?;

        let mut ctx = SpendContext::new();

        if !fee_coins.is_empty() {
            let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();

            let fee_change: u64 = (fee_total - fee as u128)
                .try_into()
                .expect("change amount overflow");

            let mut fee_conditions =
                Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

            if fee > 0 {
                fee_conditions = fee_conditions.reserve_fee(fee);
            }

            if fee_change > 0 {
                fee_conditions = fee_conditions.create_coin(derivations[0], fee_change, None);
            }

            self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
                .await?;
        }

        let mut cat_spends = Vec::new();

        for cat in cats {
            let mut conditions = Conditions::new();

            for &derivation in &derivations {
                if remaining_count == 0 {
                    break;
                }

                let amount: u64 = (max_individual_amount as u128)
                    .min(remaining_amount)
                    .try_into()
                    .expect("output amount overflow");

                remaining_amount -= amount as u128;

                let hint = ctx.hint(derivation)?;
                conditions = conditions.create_coin(derivation, amount, Some(hint));

                remaining_count -= 1;
            }

            cat_spends.push((cat, conditions));
        }

        self.spend_cat_coins(&mut ctx, cat_spends.into_iter())
            .await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::TestWallet;

    #[test(tokio::test)]
    async fn test_cat_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(100).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(100, 0, None, false, true).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let mut cats = test.wallet.db.spendable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 1);

        let cat = test
            .wallet
            .db
            .cat_coin(cats.remove(0).coin.coin_id())
            .await?
            .expect("missing cat");
        let coin_spends = test.wallet.split_cat(vec![cat], 2, 0, false, true).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let cats = test.wallet.db.spendable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 2);

        let mut cat_coins = Vec::with_capacity(cats.len());
        for cat in cats {
            cat_coins.push(
                test.wallet
                    .db
                    .cat_coin(cat.coin.coin_id())
                    .await?
                    .expect("missing cat"),
            );
        }
        let coin_spends = test.wallet.combine_cat(cat_coins, 0, false, true).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        Ok(())
    }
}
