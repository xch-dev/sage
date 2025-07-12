use serde::{Deserialize, Serialize};

use crate::{AddressKind, Amount, Asset};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransactionRecord {
    pub height: u32,
    pub timestamp: Option<u32>,
    pub spent: Vec<TransactionCoinRecord>,
    pub created: Vec<TransactionCoinRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransactionCoinRecord {
    pub coin_id: String,
    pub amount: Amount,
    pub address: Option<String>,
    pub address_kind: AddressKind,
    pub asset: Asset,
}
