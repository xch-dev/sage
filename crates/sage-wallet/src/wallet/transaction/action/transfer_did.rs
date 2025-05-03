use chia::protocol::Bytes32;

use crate::{Action, Id, Summary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferDidAction {
    pub did_id: Id,
    pub puzzle_hash: Bytes32,
}

impl Action for TransferDidAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_dids.insert(self.did_id);
    }
}
