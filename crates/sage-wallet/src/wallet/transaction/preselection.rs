use std::collections::{HashMap, HashSet};

use crate::{Wallet, WalletError};

use super::{Action, Id, TransactionConfig};

#[derive(Debug, Default, Clone)]
pub struct Preselection {
    pub created_xch: u64,
    pub created_cats: HashMap<Id, u64>,
    pub created_nfts: HashSet<Id>,
    pub created_dids: HashSet<Id>,
    pub created_options: HashSet<Id>,
    pub spent_xch: u64,
    pub spent_cats: HashMap<Id, u64>,
    pub spent_nfts: HashSet<Id>,
    pub spent_dids: HashSet<Id>,
    pub spent_options: HashSet<Id>,
}

impl Preselection {
    pub fn new(fee: u64) -> Self {
        Self {
            spent_xch: fee,
            ..Default::default()
        }
    }
}

impl Wallet {
    pub fn preselect(&self, tx: &TransactionConfig) -> Result<Preselection, WalletError> {
        let mut preselection = Preselection::new(tx.fee);

        for (index, action) in tx.actions.iter().enumerate() {
            action.preselect(&mut preselection, index);
        }

        Ok(preselection)
    }
}
