use chia::protocol::Bytes32;

use crate::{Action, Id, Preselection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferNftAction {
    pub nft_id: Id,
    pub puzzle_hash: Bytes32,
}

impl Action for TransferNftAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_nfts.insert(self.nft_id);
    }
}
