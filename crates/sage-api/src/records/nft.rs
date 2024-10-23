use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NftRecord {
    pub launcher_id: String,
    pub launcher_id_hex: String,
    pub owner_did: Option<String>,
    pub coin_id: String,
    pub address: String,
    pub royalty_address: String,
    pub royalty_percent: String,
    pub data_uris: Vec<String>,
    pub data_hash: Option<String>,
    pub metadata_uris: Vec<String>,
    pub metadata_hash: Option<String>,
    pub license_uris: Vec<String>,
    pub license_hash: Option<String>,
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
    pub data_mime_type: Option<String>,
    pub data: Option<String>,
    pub metadata: Option<String>,
    pub created_height: Option<u32>,
    pub create_transaction_id: Option<String>,
}
