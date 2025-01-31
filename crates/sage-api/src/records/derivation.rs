use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DerivationRecord {
    pub index: u32,
    pub public_key: String,
    pub address: String,
}
