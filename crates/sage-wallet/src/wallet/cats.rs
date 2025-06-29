use chia::{
    bls::PublicKey,
    protocol::{Bytes, Bytes32, CoinSpend},
    puzzles::cat::EverythingWithSignatureTailArgs,
};
use chia_wallet_sdk::driver::{Action, Id, Spend, SpendContext};
use clvmr::NodePtr;

use crate::WalletError;

use super::{memos::calculate_memos, Wallet};

impl Wallet {
    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
        multi_issuance_key: Option<PublicKey>,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let mut ctx = SpendContext::new();

        let issue_cat = if let Some(public_key) = multi_issuance_key {
            let tail = ctx.curry(EverythingWithSignatureTailArgs::new(public_key))?;
            let tail_spend = Spend::new(tail, NodePtr::NIL);
            Action::issue_cat(tail_spend, None, amount)
        } else {
            Action::single_issue_cat(None, amount)
        };
        let actions = vec![Action::fee(fee), issue_cat];
        let outputs = self.spend(&mut ctx, vec![], &actions).await?;

        Ok((ctx.take(), outputs.cats[&Id::New(1)][0].info.asset_id))
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
        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for (puzzle_hash, amount) in amounts {
            let memos = calculate_memos(&mut ctx, puzzle_hash, include_hint, memos.clone())?;
            actions.push(Action::send(
                Id::Existing(asset_id),
                puzzle_hash,
                amount,
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

    use crate::TestWallet;

    #[test(tokio::test)]
    async fn test_send_cat() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1500).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 500);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 1);
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

        assert_eq!(test.wallet.db.xch_balance().await?, 0);
        assert_eq!(test.wallet.db.spendable_xch_coins().await?.len(), 0);
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(test.wallet.db.spendable_cat_coins(asset_id).await?.len(), 1);

        Ok(())
    }
}
