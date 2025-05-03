use chia::protocol::Bytes32;
use chia_wallet_sdk::types::Conditions;

use crate::{Action, Id, Lineation, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    fn lineate(&self, lineation: &mut Lineation<'_>, _index: usize) -> Result<(), WalletError> {
        let did = lineation.dids[&self.did_id];

        let p2 = lineation
            .p2
            .get(&did.info.p2_puzzle_hash)
            .ok_or(WalletError::MissingDerivation(did.info.p2_puzzle_hash))?;

        lineation.dids[&self.did_id] =
            did.transfer(lineation.ctx, p2, self.puzzle_hash, Conditions::new())?;

        Ok(())
    }
}
