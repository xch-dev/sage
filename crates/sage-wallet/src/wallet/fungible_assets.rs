use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::{
    driver::{Cat, SpendContext},
    types::Conditions,
};

use crate::WalletError;

use super::{Hint, Id, SendAction, SpendAction, TransactionConfig, Wallet};

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
                ))
            })
            .collect();

        self.transact(actions, fee).await
    }

    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
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

        let (mut conditions, eve) =
            Cat::single_issuance_eve(&mut ctx, coins[0].coin_id(), amount, eve_conditions)?;

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), eve.asset_id))
    }

    pub async fn combine(
        &self,
        coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        self.transact_with_coin_ids(coin_ids, Vec::new(), fee).await
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
            )));

            remaining_count -= 1;
        }

        self.transact_preselected(&mut ctx, &mut TransactionConfig::new(actions, fee))
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

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, false, true).await?;
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

        let (coin_spends, asset_id) = test.wallet.issue_cat(100, 0, false, true).await?;
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
}
