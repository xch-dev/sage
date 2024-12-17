use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Amount, CoinSpendJson, TransactionSummary};

use super::Assets;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MintOption {
    pub requested_assets: Assets,
    pub offered_assets: Assets,
    pub fee: Amount,
    pub expires_at_second: u64,
    pub did_id: String,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MintOptionResponse {
    pub summary: TransactionSummary,
    pub coin_spends: Vec<CoinSpendJson>,
}
