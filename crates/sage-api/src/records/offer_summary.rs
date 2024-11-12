use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Amount, AssetKind};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OfferSummary {
    pub fee: Amount,
    pub offered: Vec<OfferedCoin>,
    pub requested: Vec<RequestedAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OfferedCoin {
    pub coin_id: String,
    pub offered_amount: Amount,
    #[serde(flatten)]
    pub kind: AssetKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RequestedAsset {
    pub amount: Amount,
    #[serde(flatten)]
    pub kind: AssetKind,
}
