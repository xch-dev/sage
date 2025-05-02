use crate::{Action, Preselection};

/// This will create a new DID at the change puzzle hash specified
/// in the transaction config. It can be immediately spent if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Action for CreateDidAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        preselection.spent_xch += 1;
    }
}
