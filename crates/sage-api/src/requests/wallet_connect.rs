use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilterUnlockedCoins {
    pub coin_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FilterUnlockedCoinsResponse {
    pub coin_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetCoins {
    #[serde(default, rename = "type")]
    pub kind: Option<AssetCoinType>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub included_locked: Option<bool>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

pub type GetAssetCoinsResponse = Vec<SpendableCoin>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AssetCoinType {
    Cat,
    Did,
    Nft,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SpendableCoin {
    pub coin: Coin,
    pub coin_name: String,
    pub puzzle: String,
    pub confirmed_block_index: u32,
    pub locked: bool,
    pub lineage_proof: Option<LineageProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub struct Coin {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LineageProof {
    pub parent_name: Option<String>,
    pub inner_puzzle_hash: Option<String>,
    pub amount: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SignMessageWithPublicKey {
    pub message: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SignMessageWithPublicKeyResponse {
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendTransactionImmediately {
    pub spend_bundle: SpendBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendTransactionImmediatelyResponse {
    pub status: u8,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub struct CoinSpend {
    pub coin: Coin,
    pub puzzle_reveal: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub struct SpendBundle {
    pub coin_spends: Vec<CoinSpend>,
    pub aggregated_signature: String,
}
