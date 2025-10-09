use serde::{Deserialize, Serialize};

/// Filter unlocked coins from a list
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "WalletConnect",
        description = "Filter a list of coins to only include unlocked ones."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilterUnlockedCoins {
    /// Coin IDs to filter
    pub coin_ids: Vec<String>,
}

/// Response with unlocked coin IDs
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FilterUnlockedCoinsResponse {
    /// List of unlocked coin IDs
    pub coin_ids: Vec<String>,
}

/// Get spendable coins for an asset
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "WalletConnect",
        description = "Get spendable coins for a specific asset type."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct GetAssetCoins {
    /// Type of asset coin
    #[serde(default, rename = "type")]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub kind: Option<AssetCoinType>,
    /// Asset ID to filter by
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
    /// Whether to include locked coins
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub included_locked: Option<bool>,
    /// Pagination offset
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub offset: Option<u32>,
    /// Number of results to return
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub limit: Option<u32>,
}

pub type GetAssetCoinsResponse = Vec<SpendableCoin>;

/// Type of asset coin
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub enum AssetCoinType {
    Cat,
    Did,
    Nft,
}

/// Spendable coin details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SpendableCoin {
    /// Coin information
    pub coin: Coin,
    /// Coin name (ID)
    pub coin_name: String,
    /// Puzzle reveal
    pub puzzle: String,
    /// Block height where coin was confirmed
    pub confirmed_block_index: u32,
    /// Whether the coin is locked
    pub locked: bool,
    /// Optional lineage proof for CAT coins
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub lineage_proof: Option<LineageProof>,
}

/// Coin structure
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct Coin {
    /// Parent coin info
    pub parent_coin_info: String,
    /// Puzzle hash
    pub puzzle_hash: String,
    /// Amount in mojos
    pub amount: u64,
}

/// Lineage proof for CAT coins
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct LineageProof {
    /// Parent coin name
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub parent_name: Option<String>,
    /// Inner puzzle hash
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub inner_puzzle_hash: Option<String>,
    /// Amount
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub amount: Option<u64>,
}

/// Sign a message with a public key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "WalletConnect",
        description = "Sign a message using a specific public key."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignMessageWithPublicKey {
    /// Message to sign
    pub message: String,
    /// Public key to use for signing
    pub public_key: String,
}

/// Response with message signature
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignMessageWithPublicKeyResponse {
    /// Signature
    pub signature: String,
}

/// Send a transaction immediately
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "WalletConnect",
        description = "Send a transaction immediately without validation."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendTransactionImmediately {
    /// Spend bundle to send
    pub spend_bundle: SpendBundle,
}

/// Response with transaction status
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendTransactionImmediatelyResponse {
    /// Status code
    pub status: u8,
    /// Optional error message
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub error: Option<String>,
}

/// Coin spend structure
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct CoinSpend {
    /// Coin being spent
    pub coin: Coin,
    /// Puzzle reveal
    pub puzzle_reveal: String,
    /// Solution
    pub solution: String,
}

/// Spend bundle structure
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct SpendBundle {
    /// Coin spends in the bundle
    pub coin_spends: Vec<CoinSpend>,
    /// Aggregated signature
    pub aggregated_signature: String,
}

/// Sign a message by address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "WalletConnect",
        description = "Sign a message using the key associated with an address."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignMessageByAddress {
    /// Message to sign
    pub message: String,
    /// Address whose key to use
    pub address: String,
}

/// Response with signed message
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "WalletConnect"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignMessageByAddressResponse {
    /// Public key used
    pub public_key: String,
    /// Signature
    pub signature: String,
}
