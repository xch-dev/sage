use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PendingTransactionRecord {
    pub transaction_id: String,
    pub fee: Amount,
    pub submitted_at: Option<String>,
    pub expiration_height: Option<u32>,
}
