use serde::{Deserialize, Serialize};

use crate::{Amount, Asset};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OptionRecord {
    pub launcher_id: String,
    pub name: Option<String>,
    pub visible: bool,
    pub coin_id: String,
    pub address: String,
    pub amount: Amount,
    pub underlying_asset: Asset,
    pub underlying_amount: Amount,
    pub underlying_coin_id: String,
    pub strike_asset: Asset,
    pub strike_amount: Amount,
    pub expiration_seconds: u64,
    pub created_height: Option<u32>,
    pub created_timestamp: Option<u64>,
}
