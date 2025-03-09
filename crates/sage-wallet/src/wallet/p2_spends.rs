use std::mem;

use chia::protocol::Coin;
use chia_wallet_sdk::{
    driver::{SpendContext, StandardLayer},
    types::Conditions,
};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Spends the given coins individually with the given conditions. No outputs are created automatically.
    pub(crate) async fn spend_p2_coins_separately(
        &self,
        ctx: &mut SpendContext,
        coins: impl Iterator<Item = (Coin, Conditions)>,
    ) -> Result<(), WalletError> {
        for (coin, conditions) in coins {
            // We need to figure out what the synthetic public key is for this p2 coin.
            let synthetic_key = self.db.synthetic_key(coin.puzzle_hash).await?;

            // Create the standard p2 layer for the key.
            let p2 = StandardLayer::new(synthetic_key);

            // Spend the coin with the given conditions.
            p2.spend(ctx, coin, conditions)?;
        }

        Ok(())
    }

    /// Spends the coins with the first coin producing all of the output conditions.
    /// The other coins assert that the first coin is spent within the transaction.
    /// This prevents the first coin from being removed from the transaction to steal the funds.
    pub(crate) async fn spend_p2_coins(
        &self,
        ctx: &mut SpendContext,
        coins: Vec<Coin>,
        mut conditions: Conditions,
    ) -> Result<(), WalletError> {
        let first_coin_id = coins[0].coin_id();

        self.spend_p2_coins_separately(
            ctx,
            coins.into_iter().enumerate().map(|(i, coin)| {
                let conditions = if i == 0 {
                    mem::take(&mut conditions)
                } else {
                    Conditions::new().assert_concurrent_spend(first_coin_id)
                };
                (coin, conditions)
            }),
        )
        .await
    }
}
