use std::mem;

use chia::{
    protocol::Coin,
    puzzles::cat::{CatArgs, GenesisByCoinIdTailArgs},
};
use chia_wallet_sdk::driver::{Cat, SpendContext};
use clvmr::NodePtr;

use crate::{Action, AssetCoin, AssetCoinExt, FungibleAsset, Id, Spends, Summary, WalletError};

/// This will create a new single-issuance CAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssueCatAction {
    /// The amount of the CAT to issue.
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

        let mut eve_item = AssetCoin::new(eve, parent.p2);

        eve_item.conditions = eve_item.conditions.run_cat_tail(tail, NodePtr::NIL);

        spends
            .cats
            .entry(Id::New(index))
            .or_insert_with(|| FungibleAsset::new(Vec::new(), true))
            .items
            .push(eve_item);

        parent.conditions =
            mem::take(&mut parent.conditions).create_coin(puzzle_hash, self.amount, None);

        Ok(())
    }
}
