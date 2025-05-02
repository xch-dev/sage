use chia::protocol::Bytes32;

use crate::{Action, Id, Preselection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferDidAction {
    pub did_id: Id,
    pub puzzle_hash: Bytes32,
}

impl Action for TransferDidAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_dids.insert(self.did_id);
    }
}
