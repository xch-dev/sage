use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenRecord {
    pub asset_id: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub precision: u8,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
    pub balance: Amount,
    pub revocation_address: Option<String>,
}
