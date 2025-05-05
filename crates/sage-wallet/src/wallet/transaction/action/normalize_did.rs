use chia::{
    clvm_utils::tree_hash_atom,
    protocol::{Bytes32, Coin},
    puzzles::{singleton::SingletonArgs, Proof},
};
use chia_wallet_sdk::{
    driver::{Did, SpendContext},
    types::Conditions,
};

use crate::{Action, AssetCoin, AssetSpend, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeDidAction {
    pub did_id: Id,
}

impl NormalizeDidAction {
    pub fn new(did_id: Id) -> Self {
        Self { did_id }
    }
}

impl Action for NormalizeDidAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_dids.insert(self.did_id);
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .dids
            .get_mut(&self.did_id)
            .ok_or(WalletError::MissingAsset)?;

        let (spend, did) = item.did()?;

        let mut new_info = did.info;
        new_info.recovery_list_hash = Some(Bytes32::from(tree_hash_atom(&[])));
        let new_inner_puzzle_hash = new_info.inner_puzzle_hash();

        let memos = ctx.hint(did.info.p2_puzzle_hash)?;

        did.spend_with(
            ctx,
            &spend.p2,
            Conditions::new().create_coin(
                new_inner_puzzle_hash.into(),
                did.coin.amount,
                Some(memos),
            ),
        )?;

        let did = Did {
            coin: Coin::new(
                did.coin.coin_id(),
                SingletonArgs::curry_tree_hash(new_info.launcher_id, new_inner_puzzle_hash).into(),
                did.coin.amount,
            ),
            proof: Proof::Lineage(did.child_lineage_proof()),
            info: new_info,
        };

        let new_did = did.update(ctx, &spend.p2, spend.conditions.clone())?;

        *spend = AssetSpend::new(AssetCoin::Did(new_did), spend.p2);

        Ok(())
    }
}
