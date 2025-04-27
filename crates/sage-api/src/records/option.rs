use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OptionRecord {
    pub launcher_id: String,
    pub visible: bool,
    pub created_height: Option<u32>,
    pub coin_id: String,
    pub address: String,
}
