use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BulkMintNfts {
    pub nft_mints: Vec<NftMint>,
    pub did_id: String,
    pub fee: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NftMint {
    pub edition_number: Option<u32>,
    pub edition_total: Option<u32>,
    pub data_uris: Vec<String>,
    pub metadata_uris: Vec<String>,
    pub license_uris: Vec<String>,
    pub royalty_address: Option<String>,
    pub royalty_percent: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BulkMintNftsResponse {
    pub nft_ids: Vec<String>,
}
