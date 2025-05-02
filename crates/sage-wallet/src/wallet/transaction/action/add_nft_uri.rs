use crate::{Action, Id, Preselection};

/// Adds a single URI to a standard NFT with a metadata update.
/// If the NFT is not standard, this will raise an error at runtime.
///
/// Only one URI can be added in a coin spend, so if you want to add
/// multiple URIs, you will need to add multiple of this action. They
/// will be added in the order they are provided, one at a time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddNftUriAction {
    /// The NFT to update the metadata of.
    pub nft_id: Id,
    /// The kind of URI that is being added.
    pub kind: NftKind,
    /// The URI that is being added.
    pub uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NftKind {
    Data,
    Metadata,
    License,
}

impl Action for AddNftUriAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_nfts.insert(self.nft_id);
    }
}
