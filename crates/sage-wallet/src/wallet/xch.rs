use chia::{
    clvm_utils::ToTreeHash,
    protocol::{Bytes, Bytes32, CoinSpend},
};
use chia_wallet_sdk::driver::{Action, ClawbackV2, Id, SpendContext};

use crate::{
    wallet::memos::{calculate_memos, Hint},
    WalletError,
};

use super::Wallet;

impl Wallet {
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        memos: Vec<Bytes>,
        clawback: Option<u64>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let sender_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for (puzzle_hash, amount) in amounts {
            let clawback = clawback.map(|seconds| {
                ClawbackV2::new(sender_puzzle_hash, puzzle_hash, seconds, amount, false)
            });

            let memos = calculate_memos(
                &mut ctx,
                if let Some(clawback) = clawback {
                    Hint::Clawback(clawback)
                } else {
                    Hint::None
                },
                memos.clone(),
            )?;

            let p2_puzzle_hash = if let Some(clawback) = clawback {
                clawback.tree_hash().into()
            } else {
                puzzle_hash
            };

            actions.push(Action::send(Id::Xch, p2_puzzle_hash, amount, memos));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use test_log::test;
    use tokio::time::sleep;

    use crate::TestWallet;

    #[test(tokio::test)]
    async fn test_send_xch() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_change() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 250)], 250, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 750);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 2);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_hardened() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.hardened_puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_with_clawback_self() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let timestamp = test.new_block_with_current_time().await?;

        let coin_spends = test
            .wallet
            .send_xch(
                vec![(test.puzzle_hash, 1000)],
                0,
                vec![],
                Some(timestamp + 5),
            )
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_balance().await?, 0);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 0);

        sleep(Duration::from_secs(6)).await;
        test.new_block_with_current_time().await?;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_with_clawback_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let timestamp = alice.new_block_with_current_time().await?;

        let coin_spends = alice
            .wallet
            .send_xch(
                vec![(bob.puzzle_hash, 1000)],
                0,
                vec![],
                Some(timestamp + 5),
            )
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.xch_balance().await?, 0);
        assert_eq!(alice.wallet.db.spendable_xch_balance().await?, 0);
        assert_eq!(alice.wallet.db.spendable_xch_coins().await?.len(), 0);

        bob.wait_for_puzzles().await;

        assert_eq!(bob.wallet.db.xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.spendable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.spendable_xch_coins().await?.len(), 0);

        sleep(Duration::from_secs(6)).await;
        bob.new_block_with_current_time().await?;

        assert_eq!(bob.wallet.db.xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.spendable_xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.spendable_xch_coins().await?.len(), 1);

        let coin_spends = bob
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.xch_balance().await?, 1000);
        assert_eq!(alice.wallet.db.spendable_xch_balance().await?, 1000);
        assert_eq!(alice.wallet.db.spendable_xch_coins().await?.len(), 1);

        assert_eq!(bob.wallet.db.xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.spendable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.spendable_xch_coins().await?.len(), 0);

        Ok(())
    }
}
