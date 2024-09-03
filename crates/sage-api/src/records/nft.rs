use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NftRecord {
    pub encoded_id: String,
    pub launcher_id: String,
    pub coin_id: String,
    pub address: String,
    pub royalty_address: String,
    pub royalty_percent: String,
}
