use crate::{Action, Id, Preselection};

/// This will create a new single-issuance CAT.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IssueCatAction {
    /// The amount of the CAT to issue.
    pub amount: u64,
}

impl Action for IssueCatAction {
    fn preselect(&self, preselection: &mut Preselection, index: usize) {
        *preselection.created_cats.entry(Id::New(index)).or_insert(0) += self.amount;
        preselection.spent_xch += self.amount;
    }
}
