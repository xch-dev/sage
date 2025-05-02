use std::collections::{HashMap, HashSet};

use super::Id;

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
