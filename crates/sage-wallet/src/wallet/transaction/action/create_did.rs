use crate::{Action, Summary};

/// This will create a new DID at the change puzzle hash specified
/// in the transaction config. It can be immediately spent if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Action for CreateDidAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_xch += 1;
    }
}
