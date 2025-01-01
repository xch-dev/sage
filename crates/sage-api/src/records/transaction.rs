use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{AddressKind, Amount, AssetKind};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransactionRecord {
    pub height: u32,
    pub spent: Vec<TransactionCoin>,
    pub created: Vec<TransactionCoin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransactionCoin {
    pub coin_id: String,
    pub amount: Amount,
    pub address: Option<String>,
    pub address_kind: AddressKind,
    #[serde(flatten)]
    pub kind: AssetKind,
}
