use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::{Conditions, SpendContext};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        puzzle_hash: Bytes32,
        amount: u64,
        fee: u64,
        memos: Vec<Bytes>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let change: u64 = (selected - total)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new().create_coin(puzzle_hash, amount, memos);

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, change, Vec::new());
        }

        let mut ctx = SpendContext::new();

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use test_log::test;

    use crate::{SyncEvent, TestWallet};

    #[test(sqlx::test)]
    async fn test_send_xch(pool: SqlitePool) -> anyhow::Result<()> {
        let mut test = TestWallet::new(pool, 1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(test.puzzle_hash, 1000, 0, Vec::new(), false, true)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::CoinState).await;
        test.consume_until(SyncEvent::CoinState).await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(sqlx::test)]
    async fn test_send_xch_change(pool: SqlitePool) -> anyhow::Result<()> {
        let mut test = TestWallet::new(pool, 1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(test.puzzle_hash, 250, 250, Vec::new(), false, true)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::CoinState).await;
        test.consume_until(SyncEvent::CoinState).await;

        assert_eq!(test.wallet.db.balance().await?, 750);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 2);

        Ok(())
    }
}
