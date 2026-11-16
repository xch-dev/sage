use serde::{Deserialize, Serialize};

use crate::{Amount, CoinSpendJson, HandleConfirmation, SpendBundleJson, TransactionSummary};

/// Send XCH to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Send XCH to an address or handle, optionally with memos and a clawback.",
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
    /// Optional XCH coin IDs to spend first, for the amount and the fee. Each must be selectable: confirmed, unspent, and not pending or locked in an offer
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub coin_ids: Option<Vec<String>>,
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Send XCH to multiple addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Send the same amount of XCH to each of several addresses or handles in one transaction.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Combine multiple coins into one
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "Combine the given coins into one coin per asset.",
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
        tag = "Coins",
        description = "Split the given coins of a single asset into the given number of coins of roughly equal amounts.",
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
        tag = "Coins",
        description = "Combine the active wallet's smallest spendable XCH coins, up to max_coins of them, optionally only those of at most max_coin_amount. Returns the IDs of the coins it combines."
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
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
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
        tag = "Coins",
        description = "Combine the active wallet's smallest spendable coins of a CAT, up to max_coins of them, optionally only those of at most max_coin_amount. Returns the IDs of the coins it combines."
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
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
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
        tag = "Tokens",
        description = "Issue a new CAT with the given name, ticker and supply, optionally revocable by the issuer.",
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
    /// Whether the CAT can be revoked by the issuer
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub revocable: bool,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to automatically submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Send CAT tokens to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Send a CAT to an address or handle, optionally with memos and a clawback.",
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
    /// Optional coin IDs to spend first: coins of this CAT for the amount, XCH coins for the fee. Each must be selectable: confirmed, unspent, and not pending or locked in an offer
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub coin_ids: Option<Vec<String>>,
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Send CAT tokens to multiple addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Send the same amount of a CAT to each of several addresses or handles in one transaction.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

fn yes() -> bool {
    true
}

/// Send multiple assets in one transaction
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Send XCH and CATs to several recipients in one transaction, each with its own asset and amount.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Individual payment in a multi-send transaction
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Payments"))]
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
        description = "Create a new DID with the given name.",
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
        description = "Mint NFTs with the given metadata and royalties from a DID the wallet owns, in one transaction. Returns the new NFT IDs."
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
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
        description = "Send NFTs to an address or handle, optionally with a clawback.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Add a URI to an NFT
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Add a data, metadata or license URI to an NFT. Only NFTs using the default metadata updater, which can only append URIs, are supported.",
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
        description = "Assign NFTs to a DID the wallet owns, or remove their DID if none is given.",
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
        description = "Send DIDs to an address or handle, optionally with a clawback.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Normalize DIDs to latest state
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "Reset DIDs to an empty recovery list and one required verification, which is the form Sage creates them in. Useful for DIDs created by other wallets.",
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
        description = "Mint an option that lets its holder buy the underlying asset for the strike asset until it expires. The underlying asset is locked when it's minted. Returns the new option ID."
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
        description = "Exercise unexpired options the wallet holds, paying their strike amount to the creator and receiving their underlying assets.",
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

/// Cancel options
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Cancel unexpired options that were minted by and are still owned by this wallet, returning the underlying assets. The strike amount is not required, but a coin of any amount (including 0) is needed of the strike asset unless it is also the underlying asset, and of XCH unless it is the underlying or strike asset.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CancelOptions {
    /// Option IDs to cancel
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
        description = "Send options to an address or handle, optionally with a clawback.",
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
    /// Handle recipients the caller has reviewed, and the addresses they resolved to. A
    /// handle which resolves to a different address than the one last sent to must be
    /// confirmed here before it can be used.
    #[serde(default)]
    pub confirmed_handles: Vec<HandleConfirmation>,
}

/// Finalize the clawback for a set of coins
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Push clawback coins through to their receiver once their clawback period has ended. Anyone can finalize a clawback, so this works for coins sent or received by the wallet. Supports XCH and CAT clawbacks.",
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

/// Claim Clawback
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Payments",
        description = "Claim XCH sent to the wallet with the earlier version of clawback, once its clawback period has ended.",
        response_type = "TransactionResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClaimClawback {
    /// The coins to claim from the clawback v1 coin.
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
        description = "Sign coin spends with the active wallet's keys, returning a spend bundle. With partial set, signatures for keys the wallet doesn't have are skipped instead of failing, so that another signer can add them."
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
    /// Existing aggregated signature to add this wallet's signatures to.
    /// It's returned unchanged if it already includes them.
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub aggregated_signature: Option<String>,
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
        description = "Summarize what coin spends would do, without signing or submitting them."
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
        description = "Submit a signed spend bundle to the network, and track it as a pending transaction."
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
pub type CancelOptionsResponse = TransactionResponse;
pub type FinalizeClawbackResponse = TransactionResponse;
pub type ClaimClawbackResponse = TransactionResponse;
