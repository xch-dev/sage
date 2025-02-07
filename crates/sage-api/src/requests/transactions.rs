use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Amount, CoinSpendJson, SpendBundleJson, TransactionSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SendXch {
    pub address: String,
    pub amount: Amount,
    pub fee: Amount,
    #[serde(default)]
    pub memos: Vec<String>,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct BulkSendXch {
    pub addresses: Vec<String>,
    pub amount: Amount,
    pub fee: Amount,
    #[serde(default)]
    pub memos: Vec<String>,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineXch {
    pub coin_ids: Vec<String>,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SplitXch {
    pub coin_ids: Vec<String>,
    pub output_count: u32,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CombineCat {
    pub coin_ids: Vec<String>,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SplitCat {
    pub coin_ids: Vec<String>,
    pub output_count: u32,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct IssueCat {
    pub name: String,
    pub ticker: String,
    pub amount: Amount,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SendCat {
    pub asset_id: String,
    pub address: String,
    pub amount: Amount,
    pub fee: Amount,
    #[serde(default)]
    pub memos: Vec<String>,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct BulkSendCat {
    pub asset_id: String,
    pub addresses: Vec<String>,
    pub amount: Amount,
    pub fee: Amount,
    #[serde(default)]
    pub memos: Vec<String>,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct CreateDid {
    pub name: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct BulkMintNfts {
    pub mints: Vec<NftMint>,
    pub did_id: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct BulkMintNftsResponse {
    pub nft_ids: Vec<String>,
    pub summary: TransactionSummary,
    pub coin_spends: Vec<CoinSpendJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct NftMint {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub edition_number: Option<u32>,
    #[serde(default)]
    pub edition_total: Option<u32>,
    #[serde(default)]
    pub data_hash: Option<String>,
    #[serde(default)]
    pub data_uris: Vec<String>,
    #[serde(default)]
    pub metadata_hash: Option<String>,
    #[serde(default)]
    pub metadata_uris: Vec<String>,
    #[serde(default)]
    pub license_hash: Option<String>,
    #[serde(default)]
    pub license_uris: Vec<String>,
    #[serde(default)]
    pub royalty_address: Option<String>,
    #[serde(default)]
    pub royalty_ten_thousandths: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransferNfts {
    pub nft_ids: Vec<String>,
    pub address: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct AddNftUri {
    pub nft_id: String,
    pub uri: String,
    pub fee: Amount,
    pub kind: NftUriKind,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum NftUriKind {
    Data,
    Metadata,
    License,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct AssignNftsToDid {
    pub nft_ids: Vec<String>,
    pub did_id: Option<String>,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransferDids {
    pub did_ids: Vec<String>,
    pub address: String,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct NormalizeDids {
    pub did_ids: Vec<String>,
    pub fee: Amount,
    #[serde(default)]
    pub auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SignCoinSpends {
    pub coin_spends: Vec<CoinSpendJson>,
    #[serde(default)]
    pub auto_submit: bool,
    #[serde(default)]
    pub partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SignCoinSpendsResponse {
    pub spend_bundle: SpendBundleJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewCoinSpends {
    pub coin_spends: Vec<CoinSpendJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct TransactionResponse {
    pub summary: TransactionSummary,
    pub coin_spends: Vec<CoinSpendJson>,
}

pub type SendXchResponse = TransactionResponse;
pub type BulkSendXchResponse = TransactionResponse;
pub type CombineXchResponse = TransactionResponse;
pub type SplitXchResponse = TransactionResponse;
pub type CombineCatResponse = TransactionResponse;
pub type SplitCatResponse = TransactionResponse;
pub type IssueCatResponse = TransactionResponse;
pub type SendCatResponse = TransactionResponse;
pub type BulkSendCatResponse = TransactionResponse;
pub type CreateDidResponse = TransactionResponse;
pub type TransferNftsResponse = TransactionResponse;
pub type AddNftUriResponse = TransactionResponse;
pub type AssignNftsToDidResponse = TransactionResponse;
pub type TransferDidsResponse = TransactionResponse;
pub type NormalizeDidsResponse = TransactionResponse;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ViewCoinSpendsResponse {
    pub summary: TransactionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SubmitTransaction {
    pub spend_bundle: SpendBundleJson,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SubmitTransactionResponse {}
