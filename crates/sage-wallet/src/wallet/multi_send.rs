use std::{collections::HashMap, mem};

use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::{driver::SpendContext, types::Conditions};
use indexmap::IndexMap;

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

fn split_payments(
    payments: Vec<MultiSendPayment>,
) -> (
    Vec<MultiSendPayment>,
    IndexMap<Bytes32, Vec<MultiSendPayment>>,
) {
    let mut xch_payments = Vec::new();
    let mut cat_payments = IndexMap::new();

    for payment in payments {
        if let Some(asset_id) = payment.asset_id {
            cat_payments
                .entry(asset_id)
                .or_insert_with(Vec::new)
                .push(payment);
        } else {
            xch_payments.push(payment);
        }
    }

    (xch_payments, cat_payments)
}

impl Wallet {
    /// Sends XCH and CATs to the given puzzle hashes.
    pub async fn multi_send(
        &self,
        payments: Vec<MultiSendPayment>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if payments.is_empty() && fee == 0 {
            return Ok(Vec::new());
        }

        let (xch_payments, cat_payments) = split_payments(payments);

        let mut ctx = SpendContext::new();

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;
        let change_hint = ctx.hint(change_puzzle_hash)?;

        let xch_amount = xch_payments.iter().map(|p| p.amount as u128).sum::<u128>();
        let xch_selected_amount = xch_amount + fee as u128;
        let xch_coins = if xch_selected_amount > 0 {
            self.select_p2_coins(xch_selected_amount).await?
        } else {
            Vec::new()
        };
        let xch_total = xch_coins.iter().map(|c| c.amount as u128).sum::<u128>();
        let xch_change = (xch_total - xch_selected_amount) as u64;

        let mut selected_cat_coins = HashMap::new();

        for (&asset_id, payments) in &cat_payments {
            let cat_amount = payments.iter().map(|p| p.amount as u128).sum::<u128>();
            let cat_coins = self.select_cat_coins(asset_id, cat_amount).await?;
            let cat_total = cat_coins
                .iter()
                .map(|c| c.coin.amount as u128)
                .sum::<u128>();
            let cat_change = (cat_total - cat_amount) as u64;
            selected_cat_coins.insert(asset_id, (cat_coins, cat_change));
        }

        let mut concurrent_coin_id = if let Some(xch_coin) = xch_coins.first() {
            xch_coin.coin_id()
        } else {
            selected_cat_coins[cat_payments.last().expect("no cat payments").0].0[0]
                .coin
                .coin_id()
        };

        for (asset_id, payments) in cat_payments {
            let (cats, cat_change) = selected_cat_coins[&asset_id].clone();

            let mut conditions = Conditions::new();

            let next_concurrent_id = cats[0].coin.coin_id();

            if concurrent_coin_id != next_concurrent_id {
                conditions = conditions.assert_concurrent_spend(concurrent_coin_id);
                concurrent_coin_id = next_concurrent_id;
            }

            for payment in payments {
                let memos = calculate_memos(&mut ctx, payment.puzzle_hash, true, payment.memos)?;
                conditions = conditions.create_coin(payment.puzzle_hash, payment.amount, memos);
            }

            self.spend_cat_coins(
                &mut ctx,
                cats.into_iter().enumerate().map(|(i, cat)| {
                    if i != 0 {
                        return (cat, Conditions::new());
                    }

                    let mut conditions = mem::take(&mut conditions);

                    if cat_change > 0 {
                        conditions = conditions.create_coin(
                            change_puzzle_hash,
                            cat_change,
                            Some(change_hint),
                        );
                    }

                    (cat, conditions)
                }),
            )
            .await?;
        }

        if !xch_coins.is_empty() {
            let mut conditions = Conditions::new();

            if concurrent_coin_id != xch_coins[0].coin_id() {
                conditions = conditions.assert_concurrent_spend(concurrent_coin_id);
            }

            if fee > 0 {
                conditions = conditions.reserve_fee(fee);
            }

            if xch_change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, xch_change, None);
            }

            for payment in xch_payments {
                let memos = calculate_memos(&mut ctx, payment.puzzle_hash, false, payment.memos)?;
                conditions = conditions.create_coin(payment.puzzle_hash, payment.amount, memos);
            }

            self.spend_p2_coins(&mut ctx, xch_coins, conditions).await?;
        }

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

        let (coin_spends, bronze) = alice.wallet.issue_cat(1000, 0, None, false, true).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, silver) = alice.wallet.issue_cat(1000, 0, None, false, true).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, gold) = alice.wallet.issue_cat(1000, 0, None, false, true).await?;
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
                false,
                true,
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
            .multi_send(
                vec![MultiSendPayment::xch(bob.puzzle_hash, 500)],
                250,
                false,
                true,
            )
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
            .multi_send(
                vec![MultiSendPayment::xch(bob.puzzle_hash, 500)],
                0,
                false,
                true,
            )
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
                false,
                true,
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
