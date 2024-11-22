use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Amount, OfferSummary, SpendBundleJson, TransactionSummary};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MakeOffer {
    pub requested_assets: Assets,
    pub offered_assets: Assets,
    pub fee: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Assets {
    pub xch: Amount,
    pub cats: Vec<CatAmount>,
    pub nfts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CatAmount {
    pub asset_id: String,
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MakeOfferResponse {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TakeOffer {
    pub offer: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TakeOfferResponse {
    pub summary: TransactionSummary,
    pub spend_bundle: SpendBundleJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ViewOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ViewOfferResponse {
    pub offer: OfferSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportOfferRequest {
    pub offer: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ImportOfferResponse {}
