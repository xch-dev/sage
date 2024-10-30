use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct NftStatus {
    pub nfts: u32,
    pub visible_nfts: u32,
    pub collections: u32,
    pub visible_collections: u32,
}
