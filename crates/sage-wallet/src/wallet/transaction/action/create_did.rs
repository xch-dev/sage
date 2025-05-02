use crate::{Select, Selection, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Select for CreateDidAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        selection.spent_xch += 1;
        selection.needs_xch_parent = true;

        Ok(())
    }
}
