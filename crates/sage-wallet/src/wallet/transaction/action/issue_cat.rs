use crate::{Action, Id, Summary};

/// This will create a new single-issuance CAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IssueCatAction {
    /// The amount of the CAT to issue.
    pub amount: u64,
}

impl Action for IssueCatAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        *summary.created_cats.entry(Id::New(index)).or_insert(0) += self.amount;
        summary.spent_xch += self.amount;
    }
}
