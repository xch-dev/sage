use crate::{Action, Id, Summary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignNftAction {
    pub nft_id: Id,
    pub did_id: Option<Id>,
}

impl AssignNftAction {
    pub fn new(nft_id: Id, did_id: Option<Id>) -> Self {
        Self { nft_id, did_id }
    }
}

impl Action for AssignNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);

        if let Some(did_id) = self.did_id {
            summary.spent_dids.insert(did_id);
        }
    }
}
