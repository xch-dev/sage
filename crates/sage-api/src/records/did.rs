use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DidRecord {
    pub launcher_id: String,
    pub name: Option<String>,
    pub visible: bool,
    pub coin_id: String,
    pub address: String,
    pub amount: Amount,
    pub recovery_hash: Option<String>,
    pub created_height: Option<u32>,
}
