use chia::protocol::{Bytes, Bytes32, CoinSpend};
use chia_wallet_sdk::{
    driver::{Cat, SpendContext},
    types::Conditions,
};

use crate::WalletError;

use super::{Hint, Id, SendAction, SpendAction, TransactionConfig, Wallet};

impl Wallet {
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

    /// Sends the given amount of CAT to the given puzzle hash.
    #[allow(clippy::too_many_arguments)]
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
}
