use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::{driver::SpendContext, types::Conditions};

use crate::WalletError;

use super::{memos::calculate_memos, Wallet};

impl Wallet {
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        memos: Option<Vec<Bytes>>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let combined_amount = amounts.iter().map(|(_, amount)| amount).sum::<u64>();

        let total = combined_amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let change: u64 = (selected - total)
            .try_into()
            .expect("change amount overflow");

        let mut ctx = SpendContext::new();

        let mut conditions = Conditions::new();

        for (puzzle_hash, amount) in amounts {
            conditions = conditions.create_coin(
                puzzle_hash,
                amount,
                calculate_memos(&mut ctx, puzzle_hash, false, memos.clone())?,
            );
        }

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, change, None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

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
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, None, false, true)
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
            .send_xch(vec![(test.puzzle_hash, 250)], 250, None, false, true)
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
            .send_xch(vec![(test.hardened_puzzle_hash, 1000)], 0, None, true, true)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, None, false, true)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }
}
