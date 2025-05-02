use crate::{Id, Select, Selection, WalletError};

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

impl Select for AddNftUriAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        // We need to spend the NFT to update its metadata.
        selection.spent_nfts.insert(self.nft_id);

        Ok(())
    }
}
