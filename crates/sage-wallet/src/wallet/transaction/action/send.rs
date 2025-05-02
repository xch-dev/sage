use chia::protocol::{Bytes, Bytes32};

use crate::{Id, Select, Selection, WalletError};

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

impl Select for SendAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        let amount: i64 = self.amount.try_into()?;

        if let Some(id) = self.asset_id {
            *selection.spent_cats.entry(id).or_insert(0) += amount;
        } else {
            selection.spent_xch += amount;
            selection.needs_xch_parent = true;
        }

        Ok(())
    }
}
