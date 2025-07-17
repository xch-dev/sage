use crate::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TokenRecord {
    pub asset_id: String,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
    pub balance: Amount,
    pub precision: u8,
    pub is_xch: bool,
}
