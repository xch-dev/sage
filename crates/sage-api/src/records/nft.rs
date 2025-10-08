use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum NftSpecialUseType {
    #[default]
    None,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftRecord {
    pub launcher_id: String,
    pub collection_id: Option<String>,
    pub collection_name: Option<String>,
    pub minter_did: Option<String>,
    pub owner_did: Option<String>,
    pub visible: bool,
    pub sensitive_content: bool,
    pub name: Option<String>,
    pub created_height: Option<u32>,
    pub coin_id: String,
    pub address: String,
    pub royalty_address: String,
    pub royalty_ten_thousandths: u16,
    pub data_uris: Vec<String>,
    pub data_hash: Option<String>,
    pub metadata_uris: Vec<String>,
    pub metadata_hash: Option<String>,
    pub license_uris: Vec<String>,
    pub license_hash: Option<String>,
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
    pub icon_url: Option<String>,
    pub created_timestamp: Option<u64>,
    pub special_use_type: Option<NftSpecialUseType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftData {
    pub blob: Option<String>,
    pub mime_type: Option<String>,
    pub hash_matches: bool,
    pub metadata_json: Option<String>,
    pub metadata_hash_matches: bool,
}
