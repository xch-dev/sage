use crate::{Select, Selection, WalletError};

/// This will create a new DID at the change puzzle hash specified
/// in the transaction config. It can be immediately spent if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Select for CreateDidAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        // We need 1 mojo to create the DID singleton.
        selection.spent_xch += 1;

        // We need an XCH parent to create the singleton launcher.
        selection.needs_xch_parent = true;

        Ok(())
    }
}
