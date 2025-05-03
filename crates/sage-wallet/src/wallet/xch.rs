use std::collections::HashSet;

use chia::protocol::{Bytes, Bytes32, Coin, CoinSpend};
use chia_wallet_sdk::{driver::SpendContext, types::Conditions, utils::select_coins};

use crate::{Hint, SendAction, SpendAction, TransactionConfig, WalletError};

use super::Wallet;

impl Wallet {
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        memos: Option<Vec<Bytes>>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let actions = amounts
            .into_iter()
            .map(|(puzzle_hash, amount)| {
                SpendAction::Send(SendAction::new(
                    None,
                    puzzle_hash,
                    amount,
                    Hint::Default,
                    memos.clone(),
                ))
            })
            .collect();

        self.transact(actions, fee).await
    }

    /// Combines multiple p2 coins into a single coin, with the given fee subtracted from the output.
    pub async fn combine_xch(
        &self,
        coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        self.transact_with_coin_ids(coin_ids, Vec::new(), fee).await
    }

    /// Splits the given XCH coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_xch(
        &self,
        coin_ids: Vec<Bytes32>,
        output_count: usize,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let preselection = self.preselect(&mut ctx, coin_ids).await?;

        if fee > preselection.xch.existing_amount {
            return Err(WalletError::InsufficientFunds);
        }

        let mut remaining_count = output_count;
        let mut remaining_amount = preselection.xch.existing_amount - fee;

        let max_individual_amount = remaining_amount.div_ceil(output_count as u64);

        let derivations_needed = output_count
            .div_ceil(preselection.xch.coins.len())
            .try_into()
            .expect("derivation count overflow");

        let derivations = self
            .p2_puzzle_hashes(derivations_needed, false, true)
            .await?;

        let mut actions = Vec::new();

        for &derivation in &derivations {
            if remaining_count == 0 {
                break;
            }

            let amount = max_individual_amount.min(remaining_amount);

            remaining_amount -= amount;

            actions.push(SpendAction::Send(SendAction::new(
                None,
                derivation,
                amount,
                Hint::Default,
                None,
            )));

            remaining_count -= 1;
        }

        self.transact_preselected(&mut ctx, &mut TransactionConfig::new(actions, fee))
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
    async fn test_send_xch() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_change() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 250)], 250, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 750);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 2);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_hardened() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.hardened_puzzle_hash, 1000)], 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_xch_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coins = test.wallet.db.spendable_coins().await?;
        let coin_spends = test
            .wallet
            .split_xch(coins.into_iter().map(|coin| coin.coin_id()).collect(), 3, 0)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 3);

        let coins = test.wallet.db.spendable_coins().await?;
        let coin_spends = test
            .wallet
            .combine_xch(coins.into_iter().map(|coin| coin.coin_id()).collect(), 0)
            .await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }
}
