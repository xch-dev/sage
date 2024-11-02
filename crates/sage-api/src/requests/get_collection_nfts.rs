use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCollectionNfts {
    pub collection_id: Option<String>,
    pub offset: u32,
    pub limit: u32,
    pub sort_mode: CollectionNftSortMode,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CollectionNftSortMode {
    Name,
    Recent,
}
