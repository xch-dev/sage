use serde::{Deserialize, Serialize};

use crate::{Amount, Asset};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferSummary {
    pub fee: Amount,
    pub maker: Vec<OfferAsset>,
    pub taker: Vec<OfferAsset>,
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferAsset {
    pub asset: Asset,
    pub amount: Amount,
    pub royalty: Amount,
    pub nft_royalty: Option<NftRoyalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct NftRoyalty {
    pub royalty_address: String,
    pub royalty_basis_points: u16,
}
