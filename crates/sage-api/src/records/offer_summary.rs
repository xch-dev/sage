use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferSummary {
    pub fee: Amount,
    pub maker: OfferAssets,
    pub taker: OfferAssets,
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferAssets {
    pub xch: OfferXch,
    pub cats: IndexMap<String, OfferCat>,
    pub nfts: IndexMap<String, OfferNft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferXch {
    pub amount: Amount,
    pub royalty: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferCat {
    pub amount: Amount,
    pub royalty: Amount,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct OfferNft {
    pub icon: Option<String>,
    pub name: Option<String>,
    pub royalty_ten_thousandths: u16,
    pub royalty_address: String,
}
