use crate::{Action, Id, Summary};

/// This will either assign an NFT to a DID, or remove the DID from an NFT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssignNftAction {
    /// The NFT to update the owner of.
    pub nft_id: Id,
    /// The DID (or lack thereof) to assign the NFT to.
    pub did_id: Option<Id>,
}

impl Action for AssignNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);

        if let Some(did_id) = self.did_id {
            summary.spent_dids.insert(did_id);
        }
    }
}
