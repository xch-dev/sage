use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NftRecord {
    pub encoded_id: String,
    pub launcher_id: String,
    pub encoded_owner_did: Option<String>,
    pub owner_did: Option<String>,
    pub coin_id: String,
    pub address: String,
    pub royalty_address: String,
    pub royalty_percent: String,
    pub data_uris: Vec<String>,
    pub data_hash: Option<String>,
    pub metadata_uris: Vec<String>,
    pub metadata_json: Option<String>,
    pub metadata_hash: Option<String>,
    pub license_uris: Vec<String>,
    pub license_hash: Option<String>,
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
}
