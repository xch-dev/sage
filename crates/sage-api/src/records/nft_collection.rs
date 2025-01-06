use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NftCollectionRecord {
    pub collection_id: String,
    pub did_id: String,
    pub metadata_collection_id: String,
    pub visible: bool,
    pub name: Option<String>,
    pub icon: Option<String>,
}
