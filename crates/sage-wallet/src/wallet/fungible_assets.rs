use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::driver::SpendContext;

use crate::WalletError;

use super::{Hint, Id, IssueCatAction, SendAction, SpendAction, TransactionConfig, Wallet};

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
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        memos: Option<Vec<Bytes>>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let actions = amounts
            .into_iter()
            .map(|(puzzle_hash, amount)| SpendAction::send_xch(puzzle_hash, amount, memos.clone()))
            .collect();

        Ok(self.transact(actions, fee).await?.coin_spends)
    }

    /// Sends the given amount of CAT to the given puzzle hash.
    pub async fn send_cat(
        &self,
        asset_id: Bytes32,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let actions = amounts
            .into_iter()
            .map(|(puzzle_hash, amount)| {
                SpendAction::Send(SendAction::new(
                    Some(Id::Existing(asset_id)),
                    puzzle_hash,
                    amount,
                    if include_hint { Hint::Yes } else { Hint::No },
                    memos.clone(),
                    None,
                ))
            })
            .collect();

        Ok(self.transact(actions, fee).await?.coin_spends)
    }

    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let result = self
            .transact(
                vec![SpendAction::IssueCat(IssueCatAction::new(amount))],
                fee,
            )
            .await?;

        Ok((
            result.coin_spends,
            *result.ids.values().next().expect("no cat"),
        ))
    }

    pub async fn combine(
        &self,
        coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        Ok(self
            .transact_with_coin_ids(coin_ids, Vec::new(), fee)
            .await?
            .coin_spends)
    }

    pub async fn split(
        &self,
        coin_ids: Vec<Bytes32>,
        output_count: usize,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if coin_ids.is_empty() {
            return Ok(Vec::new());
        }

        let coin_count = coin_ids.len();

        let mut ctx = SpendContext::new();

        let preselection = self.preselect(&mut ctx, coin_ids).await?;

        let (existing_amount, id) = preselection.cats.iter().next().map_or(
            (preselection.xch.existing_amount, None),
            |(&id, selected)| (selected.existing_amount, Some(id)),
        );

        if fee > existing_amount {
            return Err(WalletError::InsufficientFunds);
        }

        let mut remaining_count = output_count;
        let mut remaining_amount = existing_amount - fee;

        let max_individual_amount = remaining_amount.div_ceil(output_count as u64);

        let derivations_needed = output_count
            .div_ceil(coin_count)
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
                id,
                derivation,
                amount,
                Hint::Default,
                None,
                None,
            )));

            remaining_count -= 1;
        }

        let result = self
            .transact_preselected_alloc(&mut ctx, &mut TransactionConfig::new(actions, fee))
            .await?;

        Ok(result.coin_spends)
    }

    pub async fn multi_send(
        &self,
        payments: Vec<MultiSendPayment>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        Ok(self
            .transact(
                payments
                    .into_iter()
                    .map(|payment| {
                        SpendAction::Send(SendAction::new(
                            payment.asset_id.map(Id::Existing),
                            payment.puzzle_hash,
                            payment.amount,
                            Hint::Default,
                            payment.memos,
                            None,
                        ))
                    })
                    .collect(),
                fee,
            )
            .await?
            .coin_spends)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .split(coins.into_iter().map(|coin| coin.coin_id()).collect(), 3, 0)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 3);

        let coins = test.wallet.db.spendable_coins().await?;
        let coin_spends = test
            .wallet
            .combine(coins.into_iter().map(|coin| coin.coin_id()).collect(), 0)
            .await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 1000);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_cat() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1500).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.balance().await?, 500);
        assert_eq!(test.wallet.db.spendable_coins().await?.len(), 1);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_cat(asset_id, vec![(test.puzzle_hash, 750)], 0, true, None)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 2);

        let coin_spends = test
            .wallet
            .send_cat(asset_id, vec![(test.puzzle_hash, 1000)], 500, true, None)
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
    async fn test_cat_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(100).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(100, 0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let mut cats = test.wallet.db.spendable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 1);

        let cat = test
            .wallet
            .db
            .cat_coin(cats.remove(0).coin.coin_id())
            .await?
            .expect("missing cat");
        let coin_spends = test.wallet.split(vec![cat.coin.coin_id()], 2, 0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let cats = test.wallet.db.spendable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 2);

        let mut cat_coins = Vec::with_capacity(cats.len());
        for cat in cats {
            cat_coins.push(
                test.wallet
                    .db
                    .cat_coin(cat.coin.coin_id())
                    .await?
                    .expect("missing cat"),
            );
        }
        let coin_spends = test
            .wallet
            .combine(
                cat_coins
                    .into_iter()
                    .map(|cat| cat.coin.coin_id())
                    .collect(),
                0,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_multi_send() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(5000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, bronze) = alice.wallet.issue_cat(1000, 0).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, silver) = alice.wallet.issue_cat(1000, 0).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, gold) = alice.wallet.issue_cat(1000, 0).await?;
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
