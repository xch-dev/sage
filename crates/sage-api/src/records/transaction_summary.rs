use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransactionSummary {
    pub fee: Amount,
    pub inputs: Vec<Input>,
    pub coin_spends: Vec<CoinSpendJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SpendBundleJson {
    pub coin_spends: Vec<CoinSpendJson>,
    pub aggregated_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoinSpendJson {
    pub coin: CoinJson,
    pub puzzle_reveal: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoinJson {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Input {
    pub coin_id: String,
    pub amount: Amount,
    pub address: String,
    #[serde(flatten)]
    pub kind: InputKind,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Output {
    pub coin_id: String,
    pub amount: Amount,
    pub address: String,
    pub receiving: bool,
    pub burning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputKind {
    Unknown,
    Xch,
    Launcher,
    Cat {
        asset_id: String,
        name: Option<String>,
        ticker: Option<String>,
        icon_url: Option<String>,
    },
    Did {
        launcher_id: String,
        name: Option<String>,
    },
    Nft {
        launcher_id: String,
        image_data: Option<String>,
        image_mime_type: Option<String>,
        name: Option<String>,
    },
}
