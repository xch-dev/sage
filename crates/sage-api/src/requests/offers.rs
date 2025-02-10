use serde::{Deserialize, Serialize};

use crate::{Amount, OfferRecord, OfferSummary, SpendBundleJson, TransactionSummary};

use super::TransactionResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct MakeOffer {
    pub requested_assets: Assets,
    pub offered_assets: Assets,
    pub fee: Amount,
    #[serde(default)]
    pub receive_address: Option<String>,
    #[serde(default)]
    pub expires_at_second: Option<u64>,
    #[serde(default = "yes")]
    pub auto_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct Assets {
    pub xch: Amount,
    pub cats: Vec<CatAmount>,
    pub nfts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CatAmount {
    pub asset_id: String,
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct MakeOfferResponse {
    pub offer: String,
    pub offer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TakeOffer {
    pub offer: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
    #[serde(default = "yes")]
    pub auto_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TakeOfferResponse {
    pub summary: TransactionSummary,
    pub spend_bundle: SpendBundleJson,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineOffers {
    pub offers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineOffersResponse {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewOfferResponse {
    pub offer: OfferSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportOfferResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffers {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffersResponse {
    pub offers: Vec<OfferRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOfferResponse {
    pub offer: OfferRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteOfferResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CancelOffer {
    pub offer_id: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

pub type CancelOfferResponse = TransactionResponse;

fn yes() -> bool {
    true
}
