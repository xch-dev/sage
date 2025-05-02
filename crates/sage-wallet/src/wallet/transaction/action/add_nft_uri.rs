use crate::{Id, Select, Selection, WalletError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddNftUriAction {
    pub nft_id: Id,
    pub uri: String,
}

impl Select for AddNftUriAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        Ok(())
    }
}
