use std::{collections::HashMap, mem};

use chia::{
    bls::PublicKey,
    protocol::{Bytes, Bytes32, CoinSpend},
};
use chia_wallet_sdk::{
    driver::{Cat, SpendContext},
    types::Conditions,
};
use indexmap::IndexMap;

use crate::WalletError;

use super::Wallet;

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
}

impl Wallet {
    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
        multi_issuance_key: Option<PublicKey>,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let total_amount = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let hint = ctx.hint(p2_puzzle_hash)?;

        let eve_conditions = Conditions::new().create_coin(p2_puzzle_hash, amount, Some(hint));

        let (mut conditions, eve) = match multi_issuance_key {
            Some(pk) => {
                Cat::multi_issuance_eve(&mut ctx, coins[0].coin_id(), pk, amount, eve_conditions)?
            }
            None => Cat::single_issuance_eve(&mut ctx, coins[0].coin_id(), amount, eve_conditions)?,
        };

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), eve.asset_id))
    }

    /// Sends the given amount of CAT to the given puzzle hash.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_cat(
        &self,
        asset_id: Bytes32,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        include_hint: bool,
        memos: Vec<Bytes>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = if fee > 0 {
            self.select_p2_coins(fee as u128).await?
        } else {
            Vec::new()
        };

        let combined_amount = amounts.iter().map(|(_, amount)| amount).sum::<u64>();

        let cats = self
            .select_cat_coins(asset_id, combined_amount as u128)
            .await?;
        let cat_selected: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();
        let cat_change: u64 = (cat_selected - combined_amount as u128)
            .try_into()
            .expect("change amount overflow");

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut conditions = if fee_coins.is_empty() {
            Conditions::new()
        } else {
            Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id())
        };

        if fee > 0 {
            let fee_selected: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
            let fee_change: u64 = (fee_selected - fee as u128)
                .try_into()
                .expect("fee change overflow");

            conditions = conditions.reserve_fee(fee);

            if fee_change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, fee_change, None);
            }
        }

        let mut ctx = SpendContext::new();

        if !fee_coins.is_empty() {
            self.spend_p2_coins(&mut ctx, fee_coins, mem::take(&mut conditions))
                .await?;
        }

        for (puzzle_hash, amount) in amounts {
            let mut output_memos = if include_hint {
                vec![puzzle_hash.into()]
            } else {
                vec![]
            };
            output_memos.extend(memos.clone());
            let memos = ctx.memos(&output_memos)?;
            conditions = conditions.create_coin(puzzle_hash, amount, Some(memos));
        }

        let change_hint = ctx.hint(change_puzzle_hash)?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i != 0 {
                    return (cat, Conditions::new());
                }

                let mut conditions = mem::take(&mut conditions);

                if cat_change > 0 {
                    conditions =
                        conditions.create_coin(change_puzzle_hash, cat_change, Some(change_hint));
                }

                (cat, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }

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

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let xch_amount = payments
            .iter()
            .filter(|p| p.asset_id.is_none())
            .map(|p| p.amount as u128)
            .sum::<u128>();

        let xch_coins = if xch_amount + fee as u128 > 0 {
            self.select_p2_coins(xch_amount + fee as u128).await?
        } else {
            Vec::new()
        };

        let xch_total = xch_coins.iter().map(|c| c.amount as u128).sum::<u128>();
        let xch_change = xch_total - xch_amount - fee as u128;

        let mut cat_payments = IndexMap::new();
        let mut xch_payments = Vec::new();

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

        let mut cat_coins = HashMap::new();

        for (&asset_id, payments) in &cat_payments {
            let amount = payments.iter().map(|p| p.amount as u128).sum::<u128>();
            let cats = self.select_cat_coins(asset_id, amount).await?;
            cat_coins.insert(asset_id, (amount, cats));
        }

        let mut ctx = SpendContext::new();

        let mut concurrent_coin_id = if let Some(xch_coin) = xch_coins.first() {
            xch_coin.coin_id()
        } else {
            cat_coins[cat_payments.last().expect("no cat payments").0]
                .1
                .first()
                .expect("no cat coins")
                .coin
                .coin_id()
        };

        for (asset_id, payments) in cat_payments {
            let (amount, cats) = cat_coins[&asset_id].clone();
            let total = cats.iter().map(|c| c.coin.amount as u128).sum::<u128>();
            let change = (total - amount) as u64;

            let mut conditions = Conditions::new();

            let next_concurrent_id = cats[0].coin.coin_id();

            if concurrent_coin_id != next_concurrent_id {
                conditions = conditions.assert_concurrent_spend(concurrent_coin_id);
                concurrent_coin_id = next_concurrent_id;
            }

            for payment in payments {
                let mut output_memos = vec![payment.puzzle_hash.into()];
                if let Some(memos) = payment.memos {
                    output_memos.extend(memos);
                }
                let memos = ctx.memos(&output_memos)?;
                conditions =
                    conditions.create_coin(payment.puzzle_hash, payment.amount, Some(memos));
            }

            let change_hint = ctx.hint(change_puzzle_hash)?;

            self.spend_cat_coins(
                &mut ctx,
                cats.into_iter().enumerate().map(|(i, cat)| {
                    if i != 0 {
                        return (cat, Conditions::new());
                    }

                    let mut conditions = mem::take(&mut conditions);

                    if change > 0 {
                        conditions =
                            conditions.create_coin(change_puzzle_hash, change, Some(change_hint));
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
                conditions = conditions.create_coin(
                    change_puzzle_hash,
                    xch_change.try_into().expect("xch change overflow"),
                    None,
                );
            }

            for payment in xch_payments {
                let memos = if let Some(memos) = payment.memos {
                    Some(ctx.memos(&memos)?)
                } else {
                    None
                };
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
    async fn test_send_cat() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1500).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None, false, true).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 500);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_cat(
                asset_id,
                vec![(test.puzzle_hash, 750)],
                0,
                true,
                Vec::new(),
                false,
                true,
            )
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 2);

        let coin_spends = test
            .wallet
            .send_cat(
                asset_id,
                vec![(test.puzzle_hash, 1000)],
                500,
                true,
                Vec::new(),
                false,
                true,
            )
            .await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 0);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 0);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        Ok(())
    }

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

        Ok(())
    }
}
