use serde::{Deserialize, Serialize};
use specta::Type;

use crate::NftRecord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetNfts {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftsResponse {
    pub items: Vec<NftRecord>,
    pub total: u32,
}
