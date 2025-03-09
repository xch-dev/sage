use std::collections::HashSet;

use chia::protocol::{Bytes32, Coin, CoinSpend};
use chia_wallet_sdk::{driver::SpendContext, types::Conditions, utils::select_coins};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Combines multiple p2 coins into a single coin, with the given fee subtracted from the output.
    pub async fn combine_xch(
        &self,
        coins: Vec<Coin>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let change: u64 = (total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new();

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        let mut ctx = SpendContext::new();
        self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        Ok(ctx.take())
    }

    /// Splits the given XCH coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_xch(
        &self,
        coins: &[Coin],
        output_count: usize,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let mut remaining_count = output_count;
        let mut remaining_amount = total - fee as u128;

        let max_individual_amount: u64 = remaining_amount
            .div_ceil(output_count as u128)
            .try_into()
            .expect("output amount overflow");

        let derivations_needed: u32 = output_count
            .div_ceil(coins.len())
            .try_into()
            .expect("derivation count overflow");

        let derivations = self
            .p2_puzzle_hashes(derivations_needed, hardened, reuse)
            .await?;

        let mut ctx = SpendContext::new();

        self.spend_p2_coins_separately(
            &mut ctx,
            coins.iter().enumerate().map(|(i, coin)| {
                let mut conditions = Conditions::new();

                if i == 0 && fee > 0 {
                    conditions = conditions.reserve_fee(fee);
                }

                if coins.len() > 1 {
                    if i == coins.len() - 1 {
                        conditions = conditions.assert_concurrent_spend(coins[0].coin_id());
                    } else {
                        conditions = conditions.assert_concurrent_spend(coins[i + 1].coin_id());
                    }
                }

                for &derivation in &derivations {
                    if remaining_count == 0 {
                        break;
                    }

                    let amount: u64 = (max_individual_amount as u128)
                        .min(remaining_amount)
                        .try_into()
                        .expect("output amount overflow");

                    remaining_amount -= amount as u128;

                    conditions = conditions.create_coin(derivation, amount, None);

                    remaining_count -= 1;
                }

                (*coin, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    /// Creates a transaction that transfers the given coins to the given puzzle hash, minus the fee as needed.
    /// Since the parent coins are all unique, there are no coin id conflicts in the output.
    pub async fn transfer_xch(
        &self,
        coins: Vec<Coin>,
        puzzle_hash: Bytes32,
        mut fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        // Select the most optimal coins to use for the fee, to keep cost to a minimum.
        let fee_coins: HashSet<Coin> = select_coins(coins.clone(), fee as u128)?
            .into_iter()
            .collect();

        let mut ctx = SpendContext::new();

        self.spend_p2_coins_separately(
            &mut ctx,
            coins.iter().enumerate().map(|(i, coin)| {
                let conditions = if fee > 0 && fee_coins.contains(coin) {
                    // Consume as much as possible from the fee.
                    let consumed = fee.min(coin.amount);
                    fee -= consumed;

                    // If there is excess amount in this coin after the fee is paid, create a new output.
                    if consumed < coin.amount {
                        Conditions::new().create_coin(puzzle_hash, coin.amount - consumed, None)
                    } else {
                        Conditions::new()
                    }
                } else {
                    // Otherwise, just create a new output coin at the given puzzle hash.
                    Conditions::new().create_coin(puzzle_hash, coin.amount, None)
                };

                // Ensure that there is a ring of assertions for all of the coins.
                // This prevents any of them from being removed from the transaction later.
                let conditions = if coins.len() > 1 {
                    if i == coins.len() - 1 {
                        conditions.assert_concurrent_spend(coins[0].coin_id())
                    } else {
                        conditions.assert_concurrent_spend(coins[i + 1].coin_id())
                    }
                } else {
                    conditions
                };

                // The fee is reserved by one coin, even though it can come from multiple coins.
                let conditions = if i == 0 {
                    conditions.reserve_fee(fee)
                } else {
                    conditions
                };

                (*coin, conditions)
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
    async fn test_xch_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coins = test.wallet.db.spendable_coins().await?;
        let coin_spends = test.wallet.split_xch(&coins, 3, 0, false, true).await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 3);

        let coins = test.wallet.db.spendable_coins().await?;
        let coin_spends = test.wallet.combine_xch(coins, 0, false, true).await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }
}
