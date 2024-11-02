use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetNfts {
    pub offset: u32,
    pub limit: u32,
    pub sort_mode: NftSortMode,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum NftSortMode {
    Name,
    Recent,
}
