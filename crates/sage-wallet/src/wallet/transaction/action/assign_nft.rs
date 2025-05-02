use crate::{Id, Select, Selection, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssignNftAction {
    pub nft_id: Id,
    pub did_id: Id,
}

impl Select for AssignNftAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        Ok(())
    }
}
