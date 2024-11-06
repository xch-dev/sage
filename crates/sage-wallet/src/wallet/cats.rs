use chia::{
    bls::PublicKey,
    protocol::{Bytes32, CoinSpend},
};
use chia_wallet_sdk::{Cat, Conditions, SpendContext};

use crate::WalletError;

use super::Wallet;

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

        let eve_conditions = Conditions::new().create_coin(
            p2_puzzle_hash,
            amount,
            vec![p2_puzzle_hash.to_vec().into()],
        );

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
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), eve.asset_id))
    }

    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_cat(
        &self,
        asset_id: Bytes32,
        puzzle_hash: Bytes32,
        amount: u64,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_selected: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
        let fee_change: u64 = (fee_selected - fee as u128)
            .try_into()
            .expect("fee change overflow");

        let cats = self.select_cat_coins(asset_id, amount as u128).await?;
        let cat_selected: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();
        let cat_change: u64 = (cat_selected - amount as u128)
            .try_into()
            .expect("change amount overflow");

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, fee_change, Vec::new());
        }

        let mut ctx = SpendContext::new();

        self.spend_p2_coins(&mut ctx, fee_coins, conditions).await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i != 0 {
                    return (cat, Conditions::new());
                }

                let mut conditions =
                    Conditions::new().create_coin(puzzle_hash, amount, vec![puzzle_hash.into()]);

                if cat_change > 0 {
                    conditions = conditions.create_coin(
                        change_puzzle_hash,
                        cat_change,
                        vec![change_puzzle_hash.into()],
                    );
                }

                (cat, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }
}
