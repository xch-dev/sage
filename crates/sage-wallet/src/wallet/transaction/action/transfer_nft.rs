use chia::protocol::Bytes32;

use crate::{Action, Id, Summary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferNftAction {
    pub nft_id: Id,
    pub puzzle_hash: Bytes32,
}

impl Action for TransferNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);
    }
}
