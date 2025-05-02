use crate::{Id, Select, Selection, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NormalizeDidAction {
    pub did_id: Id,
}

impl Select for NormalizeDidAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        Ok(())
    }
}
