use chia::protocol::{Bytes, Bytes32};

use crate::{Action, Id, Preselection};

/// Sends an amount of a fungible asset to a given puzzle hash. This means
/// that a coin will be created at the puzzle hash and amount, but the
/// parent can be anything since there may need to be an intermediate coin
/// in between the selected coin from the wallet and the payment to prevent
/// conflicts.
///
/// If a CAT is sent, a hint will be included by default to make it possible
/// for the recipient to discover the CAT by looking up the puzzle hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SendAction {
    /// The id of the asset to send.
    pub asset_id: Option<Id>,
    /// The puzzle hash to send the asset to.
    pub puzzle_hash: Bytes32,
    /// The amount of the asset to send.
    pub amount: u64,
    /// Whether to include a hint in the transaction.
    pub include_hint: Hint,
    /// The memos to include in the transaction after the hint.
    pub memos: Option<Vec<Bytes>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hint {
    #[default]
    Default,
    Yes,
    No,
}

impl Action for SendAction {
    fn preselect(&self, preselection: &mut Preselection, _index: usize) {
        if let Some(id) = self.asset_id {
            *preselection.spent_cats.entry(id).or_insert(0) += self.amount;
        } else {
            preselection.spent_xch += self.amount;
        }
    }
}
