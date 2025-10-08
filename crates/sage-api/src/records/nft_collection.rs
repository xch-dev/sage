use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftCollectionRecord {
    pub collection_id: String,
    pub did_id: String,
    pub metadata_collection_id: String,
    pub visible: bool,
    pub name: Option<String>,
    pub icon: Option<String>,
}
