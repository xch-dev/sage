use crate::{Id, Select, Selection, WalletError};

/// This will either assign an NFT to a DID, or remove the DID from an NFT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssignNftAction {
    /// The NFT to update the owner of.
    pub nft_id: Id,
    /// The DID (or lack thereof) to assign the NFT to.
    pub did_id: Option<Id>,
}

impl Select for AssignNftAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        // We need to spend the NFT to update its owner.
        selection.spent_nfts.insert(self.nft_id);

        // We need to spend the DID to authorize the NFT ownership change.
        if let Some(did_id) = self.did_id {
            selection.spent_dids.insert(did_id);
        }

        Ok(())
    }
}
