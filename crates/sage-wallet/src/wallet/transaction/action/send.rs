use chia::protocol::{Bytes, Bytes32};
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendAction {
    pub asset_id: Option<Id>,
    pub puzzle_hash: Bytes32,
    pub amount: u64,
    pub include_hint: Hint,
    pub memos: Option<Vec<Bytes>>,
}

impl SendAction {
    pub fn new(
        asset_id: Option<Id>,
        puzzle_hash: Bytes32,
        amount: u64,
        include_hint: Hint,
        memos: Option<Vec<Bytes>>,
    ) -> Self {
        Self {
            asset_id,
            puzzle_hash,
            amount,
            include_hint,
            memos,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hint {
    #[default]
    Default,
    Yes,
    No,
}

impl Action for SendAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        if let Some(id) = self.asset_id {
            *summary.spent_cats.entry(id).or_insert(0) += self.amount;
        } else {
            summary.spent_xch += self.amount;
        }
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        if let Some(id) = self.asset_id {
            let asset = spends.cats.get_mut(&id).ok_or(WalletError::MissingAsset)?;

            asset.create_coin(
                ctx,
                self.puzzle_hash,
                self.amount,
                matches!(self.include_hint, Hint::Default | Hint::Yes),
                self.memos.clone(),
            )?;

            Ok(())
        } else {
            spends.xch.create_coin(
                ctx,
                self.puzzle_hash,
                self.amount,
                matches!(self.include_hint, Hint::Yes),
                self.memos.clone(),
            )?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpendAction, TestWallet};

    use test_log::test;

    #[test(tokio::test)]
    async fn test_action_send_xch_no_fee() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let coin_spends = alice
            .wallet
            .transact(vec![SpendAction::send_xch(bob.puzzle_hash, 300, None)], 0)
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 700);
        assert_eq!(bob.wallet.db.balance().await?, 300);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_xch_with_fee() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let coin_spends = alice
            .wallet
            .transact(vec![SpendAction::send_xch(bob.puzzle_hash, 300, None)], 100)
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 600);
        assert_eq!(bob.wallet.db.balance().await?, 300);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_xch_multiple_coins() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::send_xch(bob.puzzle_hash, 100, None),
                    SpendAction::send_xch(bob.puzzle_hash, 200, None),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 700);
        assert_eq!(bob.wallet.db.balance().await?, 300);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 2);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_xch_conflicting_coins() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::send_xch(bob.puzzle_hash, 200, None),
                    SpendAction::send_xch(bob.puzzle_hash, 200, None),
                ],
                0,
            )
            .await?
            .coin_spends;

        // An intermediate coin is created to fulfill the conflicting payment
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 600);
        assert_eq!(bob.wallet.db.balance().await?, 400);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 2);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_xch_full_amount_two_way() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let coin_spends = alice
            .wallet
            .transact(vec![SpendAction::send_xch(bob.puzzle_hash, 1000, None)], 0)
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 0);
        assert_eq!(bob.wallet.db.balance().await?, 1000);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 0);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 1);

        let coin_spends = bob
            .wallet
            .transact(
                vec![SpendAction::send_xch(alice.puzzle_hash, 1000, None)],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 1000);
        assert_eq!(bob.wallet.db.balance().await?, 0);
        assert_eq!(alice.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(bob.wallet.db.spendable_coins().await?.len(), 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_cat_no_fee() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0).await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![SpendAction::send_cat(asset_id, bob.puzzle_hash, 300, None)],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 700);
        assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 300);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_cat_with_fee() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(900, 0).await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![SpendAction::send_cat(asset_id, bob.puzzle_hash, 300, None)],
                100,
            )
            .await?
            .coin_spends;

        // The fee is paid in XCH, so we have to spend an XCH coin too
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 600);
        assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 300);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_xch_and_cat() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(500, 0).await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::send_xch(bob.puzzle_hash, 500, None),
                    SpendAction::send_cat(asset_id, bob.puzzle_hash, 500, None),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.balance().await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 0);
        assert_eq!(bob.wallet.db.balance().await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 500);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_send_cat_and_cat() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_one) = alice.wallet.issue_cat(500, 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, asset_two) = alice.wallet.issue_cat(500, 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::send_cat(asset_one, bob.puzzle_hash, 500, None),
                    SpendAction::send_cat(asset_two, bob.puzzle_hash, 500, None),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.cat_balance(asset_one).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(asset_two).await?, 0);
        assert_eq!(bob.wallet.db.cat_balance(asset_one).await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(asset_two).await?, 500);

        Ok(())
    }
}
