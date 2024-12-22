use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Amount, AssetKind};

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
    #[serde(flatten)]
    pub kind: AssetKind,
}
