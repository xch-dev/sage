use chia::clvm_utils::tree_hash_atom;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

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
        _ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .dids
            .get_mut(&self.did_id)
            .ok_or(WalletError::MissingAsset)?;

        item.set_recovery_list_hash(Some(tree_hash_atom(&[]).into()));

        Ok(())
    }
}
