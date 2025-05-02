use chia::bls::SecretKey;

use crate::{Select, Selection, WalletError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IssueCatAction {
    pub secret_key: Option<SecretKey>,
    pub amount: u64,
}

impl Select for IssueCatAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        let amount: i64 = self.amount.try_into()?;
        *selection.required_cats.entry(Id::New(index)).or_insert(0) -= amount;
        selection.required_xch += amount;
        needs_xch_parent = true;

        Ok(())
    }
}
