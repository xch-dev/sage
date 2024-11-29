use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DerivationRecord {
    pub index: u32,
    pub public_key: String,
    pub address: String,
}
