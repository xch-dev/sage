use chia::protocol::CoinSpend;
use chia_wallet_sdk::{Cat, Conditions, SpendContext};

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
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
        let cat_total: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let fee_change: u64 = (fee_total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut fee_conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            fee_conditions = fee_conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            fee_conditions = fee_conditions.create_coin(p2_puzzle_hash, fee_change, Vec::new());
        }

        let mut ctx = SpendContext::new();

        self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
            .await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i == 0 {
                    (
                        cat,
                        Conditions::new().create_coin(
                            p2_puzzle_hash,
                            cat_total.try_into().expect("output amount overflow"),
                            vec![p2_puzzle_hash.into()],
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
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
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

        let fee_change: u64 = (fee_total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut fee_conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            fee_conditions = fee_conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            fee_conditions = fee_conditions.create_coin(derivations[0], fee_change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
            .await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().map(|cat| {
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

                    conditions =
                        conditions.create_coin(derivation, amount, vec![derivation.into()]);

                    remaining_count -= 1;
                }

                (cat, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }
}
