use std::mem;

use chia::{
    protocol::Coin,
    puzzles::cat::{CatArgs, GenesisByCoinIdTailArgs},
};
use chia_wallet_sdk::driver::{Cat, SpendContext};
use clvmr::NodePtr;

use crate::{Action, AssetCoin, AssetCoinExt, FungibleAsset, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssueCatAction {
    pub amount: u64,
}

impl IssueCatAction {
    pub fn new(amount: u64) -> Self {
        Self { amount }
    }
}

impl Action for IssueCatAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        *summary.created_cats.entry(Id::New(index)).or_insert(0) += self.amount;
        summary.spent_xch += self.amount;
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        let parent_ref = spends.xch.create_from_unique_parent(ctx)?;
        let parent = spends.xch.get_mut(parent_ref)?;

        let parent_coin_id = parent.coin.coin_id();
        let tail = ctx.curry(GenesisByCoinIdTailArgs::new(parent_coin_id))?;
        let asset_id = ctx.tree_hash(tail).into();

        let inner_puzzle_hash = parent.coin.p2_puzzle_hash();
        let puzzle_hash = CatArgs::curry_tree_hash(asset_id, inner_puzzle_hash.into()).into();

        let eve = Cat::new(
            Coin::new(parent_coin_id, puzzle_hash, self.amount),
            None,
            asset_id,
            inner_puzzle_hash,
        );

        let mut eve_item = AssetCoin::new(eve, parent.p2.cleared());

        let eve_p2 = eve_item
            .p2
            .as_standard_mut()
            .ok_or(WalletError::P2Unsupported)?;

        eve_p2.conditions = mem::take(&mut eve_p2.conditions).run_cat_tail(tail, NodePtr::NIL);

        spends
            .cats
            .entry(Id::New(index))
            .or_insert_with(|| FungibleAsset::new(Vec::new()))
            .items
            .push(eve_item);

        let parent_p2 = parent
            .p2
            .as_standard_mut()
            .ok_or(WalletError::P2Unsupported)?;

        parent_p2.conditions =
            mem::take(&mut parent_p2.conditions).create_coin(puzzle_hash, self.amount, None);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpendAction, TestWallet};

    use itertools::Itertools;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_action_issue_multiple_cats() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let result = alice
            .wallet
            .transact(
                vec![SpendAction::issue_cat(500), SpendAction::issue_cat(500)],
                0,
            )
            .await?;

        // 1 spend for the original XCH coin, 1 for the intermediate coin, and 2 for the eve CATs
        // An intermediate coin is created because the CATs are created from a unique parent
        assert_eq!(result.coin_spends.len(), 4);

        let asset_ids = result.ids.into_values().collect_vec();

        alice.transact(result.coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::send_cat(asset_ids[0], bob.puzzle_hash, 500, None),
                    SpendAction::send_cat(asset_ids[1], bob.puzzle_hash, 500, None),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.cat_balance(asset_ids[0]).await?, 0);
        assert_eq!(alice.wallet.db.cat_balance(asset_ids[1]).await?, 0);
        assert_eq!(bob.wallet.db.cat_balance(asset_ids[0]).await?, 500);
        assert_eq!(bob.wallet.db.cat_balance(asset_ids[1]).await?, 500);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_issue_cat_and_send() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let result = alice
            .wallet
            .transact(
                vec![
                    SpendAction::issue_cat(1000),
                    SpendAction::send_new_cat(0, bob.puzzle_hash, 1000, None),
                ],
                0,
            )
            .await?;

        assert_eq!(result.coin_spends.len(), 2);

        let asset_id = result.ids.into_values().next().expect("no asset id");

        alice.transact(result.coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 0);
        assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 1000);

        Ok(())
    }
}
