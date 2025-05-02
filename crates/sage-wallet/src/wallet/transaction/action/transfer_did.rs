use chia::protocol::Bytes32;

use crate::{Id, Select, Selection, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferDidAction {
    pub did_id: Id,
    pub puzzle_hash: Bytes32,
}

impl Select for TransferDidAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        Ok(())
    }
}
