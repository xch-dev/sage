use chia::{
    clvm_utils::tree_hash_atom,
    protocol::{Bytes32, Coin},
    puzzles::{singleton::SingletonArgs, Proof},
};
use chia_wallet_sdk::{driver::Did, types::Conditions};

use crate::{Action, Id, Lineation, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    fn lineate(&self, lineation: &mut Lineation<'_>, _index: usize) -> Result<(), WalletError> {
        let did = lineation.dids[&self.did_id];

        let p2 = lineation
            .p2
            .get(&did.info.p2_puzzle_hash)
            .ok_or(WalletError::MissingDerivation(did.info.p2_puzzle_hash))?;

        let mut new_info = did.info;
        new_info.recovery_list_hash = Some(Bytes32::from(tree_hash_atom(&[])));
        let new_inner_puzzle_hash = new_info.inner_puzzle_hash();

        let memos = lineation.ctx.hint(did.info.p2_puzzle_hash)?;

        did.spend_with(
            lineation.ctx,
            p2,
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

        lineation.dids[&self.did_id] = did.update(lineation.ctx, p2, Conditions::new())?;

        Ok(())
    }
}
