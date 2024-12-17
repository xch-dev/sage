use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Amount, OfferRecord, OfferSummary, SpendBundleJson, TransactionSummary};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MakeOffer {
    pub requested_assets: Assets,
    pub offered_assets: Assets,
    pub fee: Amount,
    pub expires_at_second: Option<u64>,
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
    pub offer_id: String,
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
    pub transaction_id: String,
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
pub struct ImportOffer {
    pub offer: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ImportOfferResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetOffers {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOffersResponse {
    pub offers: Vec<OfferRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOfferResponse {
    pub offer: OfferRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteOffer {
    pub offer_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct DeleteOfferResponse {}
