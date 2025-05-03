mod action;
mod distribution;
mod id;
mod preselection;
mod selection;

pub use action::*;
pub use distribution::*;
pub use id::*;
pub use preselection::*;
pub use selection::*;

use chia::protocol::{Bytes32, CoinSpend};

#[derive(Debug, Default, Clone)]
pub struct TransactionConfig {
    pub actions: Vec<SpendAction>,
    pub preselected_coin_ids: Vec<Bytes32>,
    pub fee: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionResult {
    pub coin_spends: Vec<CoinSpend>,
}
