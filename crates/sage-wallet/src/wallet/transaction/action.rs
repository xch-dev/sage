mod create_did;
mod issue_cat;
mod send;

pub use create_did::*;
pub use issue_cat::*;
pub use send::*;

use crate::WalletError;

use super::{Select, Selection};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Send(SendAction),
    IssueCat(IssueCatAction),
    CreateDid(CreateDidAction),
}

impl Select for Action {
    fn select(&self, selection: &mut Selection, index: usize) -> Result<(), WalletError> {
        match self {
            Action::Send(action) => action.select(selection, index),
            Action::IssueCat(action) => action.select(selection, index),
            Action::CreateDid(action) => action.select(selection, index),
        }
    }
}
