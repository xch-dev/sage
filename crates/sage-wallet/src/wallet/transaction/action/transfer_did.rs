use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferDidAction {
    pub did_id: Id,
    pub puzzle_hash: Bytes32,
}

impl TransferDidAction {
    pub fn new(did_id: Id, puzzle_hash: Bytes32) -> Self {
        Self {
            did_id,
            puzzle_hash,
        }
    }
}

impl Action for TransferDidAction {
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

        item.set_p2_puzzle_hash(self.puzzle_hash);

        Ok(())
    }
}
