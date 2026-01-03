use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CoinRecord {
    pub coin_id: String,
    pub address: String,
    pub amount: Amount,
    pub transaction_id: Option<String>,
    pub offer_id: Option<String>,
    pub clawback_timestamp: Option<u64>,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
    pub spent_timestamp: Option<u64>,
    pub created_timestamp: Option<u64>,
    pub asset_hash: Option<String>,
}
