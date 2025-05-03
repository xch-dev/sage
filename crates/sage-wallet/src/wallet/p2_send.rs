use chia::protocol::{Bytes, Bytes32, CoinSpend};

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

        Ok(self
            .transact(&TransactionConfig::new(actions, fee))
            .await?
            .coin_spends)
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
}
