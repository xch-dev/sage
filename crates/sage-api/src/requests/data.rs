use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    Amount, CatRecord, CoinRecord, DerivationRecord, DidRecord, NftCollectionRecord, NftData,
    NftRecord, PendingTransactionRecord, TransactionRecord, Unit,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetDerivations {
    #[serde(default)]
    pub hardened: bool,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetDerivationsResponse {
    pub derivations: Vec<DerivationRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetSyncStatus {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetSyncStatusResponse {
    pub balance: Amount,
    pub unit: Unit,
    pub synced_coins: u32,
    pub total_coins: u32,
    pub receive_address: String,
    pub burn_address: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetXchCoins {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetXchCoinsResponse {
    pub coins: Vec<CoinRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCatCoins {
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCatCoinsResponse {
    pub coins: Vec<CoinRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetCats {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCatsResponse {
    pub cats: Vec<CatRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCat {
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCatResponse {
    pub cat: Option<CatRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetDids {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetDidsResponse {
    pub dids: Vec<DidRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetMinterDidIds {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetMinterDidIdsResponse {
    pub did_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetPendingTransactions {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetPendingTransactionsResponse {
    pub transactions: Vec<PendingTransactionRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetTransactions {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTransactionsEx {
    pub offset: u32,
    pub limit: u32,
    pub ascending: bool,
    pub find_column: Option<String>,
    pub find_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTransactionsResponse {
    pub transactions: Vec<TransactionRecord>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTransactionsExResponse {
    pub transactions: Vec<TransactionRecord>,
    pub total: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetTransaction {
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTransactionResponse {
    pub transaction: TransactionRecord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetNftCollections {
    pub offset: u32,
    pub limit: u32,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftCollectionsResponse {
    pub collections: Vec<NftCollectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftCollection {
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftCollectionResponse {
    pub collection: Option<NftCollectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNfts {
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub minter_did_id: Option<String>,
    #[serde(default)]
    pub owner_did_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub offset: u32,
    pub limit: u32,
    pub sort_mode: NftSortMode,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftsResponse {
    pub nfts: Vec<NftRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum NftSortMode {
    Name,
    Recent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNft {
    pub nft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftResponse {
    pub nft: Option<NftRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftData {
    pub nft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNftDataResponse {
    pub data: Option<NftData>,
}
