use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Amount, OfferRecord, OfferSummary, SpendBundleJson, TransactionSummary};

use super::TransactionResponse;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[schema(example = json!({
    "requested_assets": {
        "xch": "1000000000000",
        "cats": [],
        "nfts": []
    },
    "offered_assets": {
        "xch": "0",
        "cats": [{"asset_id": "...", "amount": "1000000"}],
        "nfts": []
    },
    "fee": "0",
    "expires_at_second": null
}))]
pub struct MakeOffer {
    #[schema(example = "Assets being requested in the offer")]
    pub requested_assets: Assets,
    #[schema(example = "Assets being offered")]
    pub offered_assets: Assets,
    #[schema(example = "Transaction fee in mojos")]
    pub fee: Amount,
    #[schema(example = "Optional receive address")]
    #[serde(default)]
    pub receive_address: Option<String>,
    #[schema(example = "Optional expiration time in seconds since epoch")]
    #[serde(default)]
    pub expires_at_second: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct Assets {
    pub xch: Amount,
    pub cats: Vec<CatAmount>,
    pub nfts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CatAmount {
    pub asset_id: String,
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct MakeOfferResponse {
    pub offer: String,
    pub offer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TakeOffer {
    pub offer: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TakeOfferResponse {
    pub summary: TransactionSummary,
    pub spend_bundle: SpendBundleJson,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineOffers {
    pub offers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineOffersResponse {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewOfferResponse {
    pub offer: OfferSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportOfferResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffers {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffersResponse {
    pub offers: Vec<OfferRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetOfferResponse {
    pub offer: OfferRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteOfferResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CancelOffer {
    pub offer_id: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

pub type CancelOfferResponse = TransactionResponse;
