use serde::{Deserialize, Serialize};

use crate::{Amount, CoinSpendJson, FeePolicy, SpendBundleJson, TransactionSummary};

/// Send XCH to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Send XCH to a recipient address with optional fee and memos.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendXch {
    /// Recipient address
    #[cfg_attr(feature = "openapi", schema(example = "xch1..."))]
    pub address: String,
    /// Amount to send
    pub amount: Amount,
    /// Transaction fee
    pub fee: Amount,
    /// Optional memos
    #[serde(default)]
    pub memos: Vec<String>,
    /// Optional clawback timestamp (seconds since epoch)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Send XCH to multiple addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Send XCH to multiple addresses in a single transaction.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkSendXch {
    /// List of recipient addresses
    pub addresses: Vec<String>,
    /// Amount to send to each address
    pub amount: Amount,
    /// Transaction fee
    pub fee: Amount,
    /// Optional memos
    #[serde(default)]
    pub memos: Vec<String>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Combine multiple coins into one
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Combine multiple small coins into a single larger coin to reduce coin count.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Combine {
    /// Coin IDs to combine
    pub coin_ids: Vec<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Split coins into multiple smaller coins
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Split a large coin into multiple smaller coins of specified amounts.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Split {
    /// Coin IDs to split
    pub coin_ids: Vec<String>,
    /// Number of output coins
    pub output_count: u32,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Automatically combine XCH coins
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Automatically combine XCH coins based on configurable thresholds."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AutoCombineXch {
    /// Maximum number of coins to combine
    pub max_coins: u32,
    /// Optional maximum amount per coin
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub max_coin_amount: Option<Amount>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response for auto-combine XCH
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCH Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AutoCombineXchResponse {
    /// Combined coin IDs
    pub coin_ids: Vec<String>,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Automatically combine CAT coins
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "CAT Tokens",
        description = "Automatically combine CAT token coins to reduce coin count."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AutoCombineCat {
    /// Asset ID of the CAT
    pub asset_id: String,
    /// Maximum number of coins to combine
    pub max_coins: u32,
    /// Optional maximum amount per coin
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub max_coin_amount: Option<Amount>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response for auto-combine CAT
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "CAT Tokens"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AutoCombineCatResponse {
    /// Combined coin IDs
    pub coin_ids: Vec<String>,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Issue a new CAT token
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "CAT Tokens",
        description = "Issue (mint) a new CAT token with a specified supply.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IssueCat {
    /// Token name
    pub name: String,
    /// Token ticker symbol
    pub ticker: String,
    /// Initial supply amount
    pub amount: Amount,
    /// Transaction fee
    pub fee: Amount,
    /// Optional transfer fee policy for fee CAT issuance
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub fee_policy: Option<FeePolicy>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Send CAT tokens to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "CAT Tokens",
        description = "Send CAT tokens to a recipient address.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SendCat {
    /// Asset ID of the CAT
    pub asset_id: String,
    /// Recipient address
    pub address: String,
    /// Amount to send
    pub amount: Amount,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to include the CAT hint
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub include_hint: bool,
    /// Optional memos
    #[serde(default)]
    pub memos: Vec<String>,
    /// Optional clawback timestamp
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Send CAT tokens to multiple addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "CAT Tokens",
        description = "Send CAT tokens to multiple addresses in a single transaction.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkSendCat {
    /// Asset ID of the CAT
    pub asset_id: String,
    /// List of recipient addresses
    pub addresses: Vec<String>,
    /// Amount to send to each address
    pub amount: Amount,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to include the CAT hint
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub include_hint: bool,
    /// Optional memos
    #[serde(default)]
    pub memos: Vec<String>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

fn yes() -> bool {
    true
}

/// Send multiple assets in one transaction
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Send multiple different assets (XCH, CATs, NFTs) in a single transaction.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MultiSend {
    /// List of payments to make
    pub payments: Vec<Payment>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Individual payment in a multi-send transaction
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCH Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Payment {
    /// Optional asset ID (null for XCH)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
    /// Recipient address
    pub address: String,
    /// Amount to send
    pub amount: Amount,
    /// Optional memos
    #[serde(default)]
    pub memos: Vec<String>,
}

/// Create a new DID
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "Create a new DID (Decentralized Identifier) for identity and NFT minting.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateDid {
    /// DID name
    pub name: String,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Mint multiple NFTs in one transaction
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Mint multiple NFTs in a single transaction with metadata and royalties."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkMintNfts {
    /// List of NFTs to mint
    pub mints: Vec<NftMint>,
    /// DID ID for the NFT collection
    pub did_id: String,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response for bulk NFT minting
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkMintNftsResponse {
    /// List of minted NFT IDs
    pub nft_ids: Vec<String>,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Individual NFT to mint
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NftMint {
    /// Optional target address
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub address: Option<String>,
    /// Edition number
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub edition_number: Option<u32>,
    /// Total editions
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub edition_total: Option<u32>,
    /// Data hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub data_hash: Option<String>,
    /// Data URIs
    #[serde(default)]
    pub data_uris: Vec<String>,
    /// Metadata hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub metadata_hash: Option<String>,
    /// Metadata URIs
    #[serde(default)]
    pub metadata_uris: Vec<String>,
    /// License hash
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub license_hash: Option<String>,
    /// License URIs
    #[serde(default)]
    pub license_uris: Vec<String>,
    /// Royalty payment address
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub royalty_address: Option<String>,
    /// Royalty percentage in ten-thousandths (e.g., 300 = 3%)
    #[serde(default)]
    pub royalty_ten_thousandths: u16,
}

/// Transfer NFTs to a new owner
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Transfer one or more NFTs to a new owner address.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransferNfts {
    /// NFT IDs to transfer
    pub nft_ids: Vec<String>,
    /// Recipient address
    pub address: String,
    /// Transaction fee
    pub fee: Amount,
    /// Optional clawback timestamp
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Add a URI to an NFT
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Add a new URI to an NFT's metadata (for updated content or mirrors.).",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddNftUri {
    /// NFT ID
    pub nft_id: String,
    /// URI to add
    pub uri: String,
    /// Transaction fee
    pub fee: Amount,
    /// Type of URI
    pub kind: NftUriKind,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Type of NFT URI
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum NftUriKind {
    Data,
    Metadata,
    License,
}

/// Assign NFTs to a DID
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Assign NFTs to a DID for identity-based ownership tracking.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignNftsToDid {
    /// NFT IDs to assign
    pub nft_ids: Vec<String>,
    /// DID ID (null to unassign)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub did_id: Option<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Transfer DIDs to a new address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "Transfer DID ownership to a new address.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransferDids {
    /// DID IDs to transfer
    pub did_ids: Vec<String>,
    /// Recipient address
    pub address: String,
    /// Transaction fee
    pub fee: Amount,
    /// Optional clawback timestamp
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Normalize DIDs to latest state
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "Update DID records to their latest on-chain state.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NormalizeDids {
    /// DID IDs to normalize
    pub did_ids: Vec<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Asset specification for options
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Options"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OptionAsset {
    /// Asset ID (null for XCH)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
    /// Amount
    pub amount: Amount,
}

/// Mint a new option
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Mint a new option (Chia options protocol) with strike price and expiration."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MintOption {
    /// Expiration time in seconds
    pub expiration_seconds: u64,
    /// Underlying asset
    pub underlying: OptionAsset,
    /// Strike price asset
    pub strike: OptionAsset,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response for minting an option
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Options"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MintOptionResponse {
    /// ID of the minted option
    pub option_id: String,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Exercise options
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Exercise options that are in-the-money and not expired.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExerciseOptions {
    /// Option IDs to exercise
    pub option_ids: Vec<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Transfer options to another address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Transfer options to another address.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransferOptions {
    /// Option IDs to transfer
    pub option_ids: Vec<String>,
    /// Recipient address
    pub address: String,
    /// Transaction fee
    pub fee: Amount,
    /// Optional clawback timestamp
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clawback: Option<u64>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Send CAT tokens to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCH Transactions",
        description = "Finalize the clawback for a set of coins.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FinalizeClawback {
    /// The coins to finalize the clawback for
    pub coin_ids: Vec<String>,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Sign coin spends to create a transaction
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "Sign coin spends to create a valid spend bundle for a transaction."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignCoinSpends {
    /// Coin spends to sign
    pub coin_spends: Vec<CoinSpendJson>,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
    /// Whether to partially sign (for multi-signature)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub partial: bool,
}

/// Response with signed spend bundle
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SignCoinSpendsResponse {
    /// Signed spend bundle
    pub spend_bundle: SpendBundleJson,
}

/// View coin spends without signing
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "View coin spends without signing, useful for transaction inspection."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ViewCoinSpends {
    /// Coin spends to view
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Response with transaction summary
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ViewCoinSpendsResponse {
    /// Transaction summary
    pub summary: TransactionSummary,
}

/// Submit a transaction to the network
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "Submit a signed transaction (spend bundle) to the blockchain network."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubmitTransaction {
    /// Spend bundle to submit
    pub spend_bundle: SpendBundleJson,
}

/// Response for transaction submission
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SubmitTransactionResponse {}

/// Standard transaction response
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionResponse {
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

pub type SendXchResponse = TransactionResponse;
pub type BulkSendXchResponse = TransactionResponse;
pub type CombineResponse = TransactionResponse;
pub type SplitResponse = TransactionResponse;
pub type IssueCatResponse = TransactionResponse;
pub type SendCatResponse = TransactionResponse;
pub type BulkSendCatResponse = TransactionResponse;
pub type MultiSendResponse = TransactionResponse;
pub type CreateDidResponse = TransactionResponse;
pub type TransferNftsResponse = TransactionResponse;
pub type AddNftUriResponse = TransactionResponse;
pub type AssignNftsToDidResponse = TransactionResponse;
pub type TransferDidsResponse = TransactionResponse;
pub type NormalizeDidsResponse = TransactionResponse;
pub type TransferOptionsResponse = TransactionResponse;
pub type ExerciseOptionsResponse = TransactionResponse;
pub type FinalizeClawbackResponse = TransactionResponse;
