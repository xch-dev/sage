use crate::{Action, Id, Preselection};

/// This will either assign an NFT to a DID, or remove the DID from an NFT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssignNftAction {
    /// The NFT to update the owner of.
    pub nft_id: Id,
    /// The DID (or lack thereof) to assign the NFT to.
    pub did_id: Option<Id>,
}

impl Action for AssignNftAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_nfts.insert(self.nft_id);

        if let Some(did_id) = self.did_id {
            preselection.spent_dids.insert(did_id);
        }
    }
}
