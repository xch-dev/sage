use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::driver::{Action, Id, SpendContext};

use crate::WalletError;

use super::{memos::calculate_memos, Wallet};

#[derive(Debug, Clone)]
pub struct MultiSendPayment {
    pub asset_id: Option<Bytes32>,
    pub amount: u64,
    pub puzzle_hash: Bytes32,
    pub memos: Option<Vec<Bytes>>,
}

impl MultiSendPayment {
    pub fn xch(puzzle_hash: Bytes32, amount: u64) -> Self {
        Self {
            asset_id: None,
            amount,
            puzzle_hash,
            memos: None,
        }
    }

    pub fn cat(asset_id: Bytes32, puzzle_hash: Bytes32, amount: u64) -> Self {
        Self {
            asset_id: Some(asset_id),
            amount,
            puzzle_hash,
            memos: None,
        }
    }

    pub fn is_xch(&self) -> bool {
        self.asset_id.is_none()
    }

    pub fn is_cat(&self) -> bool {
        self.asset_id.is_some()
    }
}

impl Wallet {
    /// Sends XCH and CATs to the given puzzle hashes.
    pub async fn multi_send(
        &self,
        payments: Vec<MultiSendPayment>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for payment in payments {
            let memos = calculate_memos(
                &mut ctx,
                payment.puzzle_hash,
                payment.asset_id.is_some(),
                payment.memos,
            )?;
            actions.push(Action::send(
                payment.asset_id.map_or(Id::Xch, Id::Existing),
                payment.puzzle_hash,
                payment.amount,
                memos,
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::{MultiSendPayment, TestWallet};

    #[test(tokio::test)]
    async fn test_multi_send() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(5000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, bronze) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, silver) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, gold) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 2000);
        assert_eq!(alice.wallet.db.cat_balance(bronze).await?, 1000);
        assert_eq!(alice.wallet.db.cat_balance(silver).await?, 1000);
        assert_eq!(alice.wallet.db.cat_balance(gold).await?, 1000);

        let coin_spends = alice
            .wallet
            .multi_send(
                vec![
                    MultiSendPayment::cat(bronze, bob.puzzle_hash, 1000),
                    MultiSendPayment::cat(silver, bob.puzzle_hash, 500),
                    MultiSendPayment::cat(gold, bob.puzzle_hash, 100),
                ],
                0,
            )
            .await?;
        assert_eq!(coin_spends.len(), 3);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.balance().await?, 2000);
        assert_eq!(alice.wallet.db.cat_balance(bronze).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(alice.wallet.db.cat_balance(gold).await?, 900);

        assert_eq!(bob.wallet.db.balance().await?, 0);
        assert_eq!(bob.wallet.db.cat_balance(bronze).await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(gold).await?, 100);

        let coin_spends = alice
            .wallet
            .multi_send(vec![MultiSendPayment::xch(bob.puzzle_hash, 500)], 250)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 1250);
        assert_eq!(alice.wallet.db.cat_balance(bronze).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(alice.wallet.db.cat_balance(gold).await?, 900);

        assert_eq!(bob.wallet.db.balance().await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(bronze).await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(gold).await?, 100);

        let coin_spends = alice
            .wallet
            .multi_send(vec![MultiSendPayment::xch(bob.puzzle_hash, 500)], 0)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        assert_eq!(alice.wallet.db.balance().await?, 750);
        assert_eq!(alice.wallet.db.cat_balance(bronze).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(alice.wallet.db.cat_balance(gold).await?, 900);

        assert_eq!(bob.wallet.db.balance().await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(bronze).await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(silver).await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(gold).await?, 100);

        let coin_spends = alice
            .wallet
            .multi_send(
                vec![
                    MultiSendPayment::xch(bob.puzzle_hash, 350),
                    MultiSendPayment::cat(silver, bob.puzzle_hash, 500),
                    MultiSendPayment::cat(gold, bob.puzzle_hash, 900),
                ],
                400,
            )
            .await?;
        assert_eq!(coin_spends.len(), 3);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.balance().await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(bronze).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(silver).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(gold).await?, 0);

        assert_eq!(bob.wallet.db.balance().await?, 1350);
        assert_eq!(bob.wallet.db.cat_balance(bronze).await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(silver).await?, 1000);
        assert_eq!(bob.wallet.db.cat_balance(gold).await?, 1000);

        Ok(())
    }
}
