use crate::{Select, Selection, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Select for CreateDidAction {
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
