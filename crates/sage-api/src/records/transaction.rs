use serde::{Deserialize, Serialize};

use crate::{AddressKind, Amount, AssetKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransactionRecord {
    pub height: u32,
    pub timestamp: Option<u32>,
    pub spent: Vec<TransactionRecordCoin>,
    pub created: Vec<TransactionRecordCoin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransactionRecordCoin {
    pub coin_id: String,
    pub amount: Amount,
    pub address: Option<String>,
    pub address_kind: AddressKind,
    #[serde(flatten)]
    pub kind: AssetKind,
    pub precision: u8,
}
