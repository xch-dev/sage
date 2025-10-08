use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PendingTransactionRecord {
    pub transaction_id: String,
    pub fee: Amount,
    pub submitted_at: Option<u64>,
}
