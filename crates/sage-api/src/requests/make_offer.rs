use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

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
