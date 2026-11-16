use serde::{Deserialize, Serialize};

use crate::{
    Amount, CoinRecord, DerivationRecord, DidRecord, NftCollectionRecord, NftData, NftRecord,
    NftUpdaterKind, OptionRecord, PendingTransactionRecord, TokenRecord, TransactionRecord, Unit,
};

/// Validate and check an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Addresses",
        description = "Check whether an address belongs to the active wallet. Returns false for addresses that can't be decoded, rather than an error. The address's prefix isn't checked against the network."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckAddress {
    /// Address to validate
    #[cfg_attr(feature = "openapi", schema(example = "xch1..."))]
    pub address: String,
}

/// Response with address validation result
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Addresses"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CheckAddressResponse {
    /// Whether the address is valid and belongs to this wallet
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub valid: bool,
}

/// Get address derivation information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Addresses",
        description = "List the active wallet's derived addresses and their public keys, either hardened or unhardened, with pagination."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDerivations {
    /// Whether to retrieve hardened derivations
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false, example = true))]
    pub hardened: bool,
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of derivations to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
}

/// Response with derivation records
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Addresses"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDerivationsResponse {
    /// List of address derivations
    pub derivations: Vec<DerivationRecord>,
    /// Total number of derivations available
    pub total: u32,
}

/// Perform database maintenance operations
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System",
        description = "Optimize the active wallet's database and reclaim free space, optionally forcing a full vacuum, which takes longer."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PerformDatabaseMaintenance {
    /// Whether to force a full vacuum (may take longer)
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub force_vacuum: bool,
}

/// Response with maintenance operation statistics
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PerformDatabaseMaintenanceResponse {
    /// Time spent vacuuming in milliseconds
    pub vacuum_duration_ms: u64,
    /// Time spent analyzing in milliseconds
    pub analyze_duration_ms: u64,
    /// Time spent checkpointing WAL in milliseconds
    pub wal_checkpoint_duration_ms: u64,
    /// Total maintenance duration in milliseconds
    pub total_duration_ms: u64,
    /// Number of pages reclaimed by vacuum
    pub pages_vacuumed: i64,
    /// Number of WAL pages checkpointed
    pub wal_pages_checkpointed: i64,
}

/// Retrieve database statistics
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System",
        description = "Get the size and free space of the active wallet's database."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDatabaseStats {}

/// Response with database statistics
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDatabaseStatsResponse {
    /// Total pages in database
    pub total_pages: i64,
    /// Number of free pages
    pub free_pages: i64,
    /// Percentage of free space
    pub free_percentage: f64,
    /// Size of each page in bytes
    pub page_size: i64,
    /// Total database size in bytes
    pub database_size_bytes: i64,
    /// Free space in bytes
    pub free_space_bytes: i64,
    /// Number of WAL pages
    pub wal_pages: i64,
}

/// Get the current synchronization status
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System",
        description = "Get the active wallet's sync progress, along with its spendable XCH balance, receive address, derivation indexes, NFT file download progress and database size."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSyncStatus {}

/// Response with detailed sync status
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSyncStatusResponse {
    /// Current wallet selectable balance
    pub selectable_balance: Amount,
    /// Unit for balance display
    pub unit: Unit,
    /// Number of coins synced
    pub synced_coins: u32,
    /// Total coins to sync
    pub total_coins: u32,
    /// Current receive address
    pub receive_address: String,
    /// Burn address for the wallet
    pub burn_address: String,
    /// Unhardened derivation index
    pub unhardened_derivation_index: u32,
    /// Hardened derivation index
    pub hardened_derivation_index: u32,
    /// Number of NFT files checked
    pub checked_files: u32,
    /// Total NFT files to check
    pub total_files: u32,
    /// Database size in bytes
    pub database_size: u64,
}

/// Get the wallet version
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "System", description = "Get the version of Sage.")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetVersion {}

/// Response with version information
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetVersionResponse {
    /// Semantic version string
    #[cfg_attr(feature = "openapi", schema(example = "0.13.1"))]
    pub version: String,
}

/// Check if specific coins are spendable
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "Check whether all of the given coins can still be spent: they belong to the active wallet, are unspent, aren't in a pending transaction, and aren't a clawback or option that can't be spent yet. Coins locked in an open offer still count as spendable. Returns false for an empty list."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetAreCoinsSpendable {
    /// List of coin IDs to check
    pub coin_ids: Vec<String>,
}

/// Response with spendability status
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetAreCoinsSpendableResponse {
    /// Whether all coins are spendable
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub spendable: bool,
}

/// Get the count of spendable coins
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "Count the active wallet's spendable coins of an asset (XCH if none is given)."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSpendableCoinCount {
    /// Optional asset ID to filter by (null for XCH)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
}

/// Response with coin count
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSpendableCoinCountResponse {
    /// Number of spendable coins
    pub count: u32,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CoinSortMode {
    CoinId,
    Amount,
    #[default]
    CreatedHeight,
    SpentHeight,
    ClawbackTimestamp,
    ClawbackVersion,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CoinFilterMode {
    All,
    #[default]
    Selectable,
    Owned,
    Spent,
    Clawback,
}

/// List coins with filtering and pagination
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "List the active wallet's coins of an asset (XCH if none is given), with filtering, sorting and pagination."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCoins {
    /// Optional asset ID to filter by
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of coins to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
    /// Sort mode
    #[serde(default)]
    pub sort_mode: CoinSortMode,
    /// Filter mode
    #[serde(default)]
    pub filter_mode: CoinFilterMode,
    /// Sort in ascending order
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub ascending: bool,
}

/// Response with coin list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCoinsResponse {
    /// List of coins
    pub coins: Vec<CoinRecord>,
    /// Total number of coins available
    pub total: u32,
}

/// Retrieve specific coins by their IDs
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "Get the active wallet's coins with the given IDs. Coins it doesn't know about are left out."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCoinsByIds {
    /// List of coin IDs to retrieve
    pub coin_ids: Vec<String>,
}

/// Response with requested coins
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCoinsByIdsResponse {
    /// List of coins matching the requested IDs
    pub coins: Vec<CoinRecord>,
}

/// Get all known CAT tokens
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Tokens",
        description = "List every CAT the active wallet knows about, including ones it has never held coins of, with their balances."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetAllCats {}

/// Response with all known CAT tokens
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Tokens"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetAllCatsResponse {
    /// List of all CAT tokens
    pub cats: Vec<TokenRecord>,
}

/// Get CAT tokens in wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Tokens",
        description = "List the CATs the active wallet has received coins of, including ones it has since spent, with their balances."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCats {}

/// Response with CAT tokens
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Tokens"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetCatsResponse {
    pub cats: Vec<TokenRecord>,
}

/// Get detailed token information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Tokens",
        description = "Get a token and the active wallet's balance of it, by CAT asset ID, or XCH if none is given."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetToken {
    /// Asset ID of the token (null for XCH)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub asset_id: Option<String>,
}

/// Response with token details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Tokens"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTokenResponse {
    /// Token information if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub token: Option<TokenRecord>,
}

/// List all DIDs in the wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "DIDs", description = "List the DIDs the active wallet owns.")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDids {}

/// Response with DID list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "DIDs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetDidsResponse {
    /// List of DIDs
    pub dids: Vec<DidRecord>,
}

/// Get minter DIDs with pagination
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "DIDs",
        description = "List the DIDs that minted the active wallet's NFTs, with pagination."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetMinterDidIds {
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of DID IDs to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
}

/// Response with minter DID IDs
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "DIDs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetMinterDidIdsResponse {
    /// List of minter DID IDs
    pub did_ids: Vec<String>,
    /// Total number of minter DIDs
    pub total: u32,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum OptionSortMode {
    #[default]
    Name,
    CreatedHeight,
    ExpirationSeconds,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum OptionFilter {
    /// Options currently held by the wallet
    #[default]
    Owned,
    /// Options minted by the wallet, including ones that have since been transferred
    Created,
    /// Both owned and created options
    All,
}

/// List options with filtering
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "List the options the active wallet holds or minted, with searching, sorting and pagination."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOptions {
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of options to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
    /// Sort mode
    #[serde(default)]
    pub sort_mode: OptionSortMode,
    /// Sort in ascending order
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub ascending: bool,
    /// Optional search value
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub find_value: Option<String>,
    /// Include hidden options
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub include_hidden: bool,
    /// Which options to list (defaults to owned options)
    #[serde(default)]
    pub filter: OptionFilter,
}

/// Response with options list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Options"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOptionsResponse {
    /// List of options
    pub options: Vec<OptionRecord>,
    /// Total number of options
    pub total: u32,
}

/// Get a specific option
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Options",
        description = "Get one of the active wallet's options by its option ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOption {
    /// Option ID
    pub option_id: String,
}

/// Response with option details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Options"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetOptionResponse {
    /// Option information if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub option: Option<OptionRecord>,
}

/// Get pending transactions
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "List the active wallet's transactions that have been submitted but not yet confirmed."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetPendingTransactions {}

/// Get a specific transaction by height
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "Get the active wallet's transaction record for a block height, which includes every coin it saw created or spent in that block."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTransaction {
    /// Transaction height/ID
    pub height: u32,
}

/// Response with transaction details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTransactionResponse {
    /// Transaction if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub transaction: Option<TransactionRecord>,
}

/// Response with pending transactions
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetPendingTransactionsResponse {
    /// List of pending transactions
    pub transactions: Vec<PendingTransactionRecord>,
}

/// List transactions with filtering
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "List the active wallet's confirmed transactions, one per block height, with searching and pagination."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTransactions {
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of transactions to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
    /// Sort in ascending order
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub ascending: bool,
    /// Optional search value
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub find_value: Option<String>,
}

/// Response with transactions list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Transactions"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetTransactionsResponse {
    /// List of transactions
    pub transactions: Vec<TransactionRecord>,
    /// Total number of transactions
    pub total: u32,
}

/// List NFT collections
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "List the collections of the active wallet's NFTs, with pagination."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftCollections {
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of collections to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
    /// Include hidden collections
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub include_hidden: bool,
}

/// Response with NFT collections
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftCollectionsResponse {
    /// List of NFT collections
    pub collections: Vec<NftCollectionRecord>,
    /// Total number of collections
    pub total: u32,
}

/// Get a specific NFT collection
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Get a collection by ID, or the placeholder for NFTs without a collection if no ID is given."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftCollection {
    /// Collection ID (null for uncollected NFTs)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub collection_id: Option<String>,
}

/// Response with NFT collection details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftCollectionResponse {
    /// Collection if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub collection: Option<NftCollectionRecord>,
}

/// List NFTs with filtering
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "List the active wallet's NFTs, optionally limited to a collection, a minter DID or an owner DID, with searching, sorting and pagination."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNfts {
    /// Filter by collection ID
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub collection_id: Option<String>,
    /// Filter by minter DID
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub minter_did_id: Option<String>,
    /// Filter by owner DID
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub owner_did_id: Option<String>,
    /// Filter by name search
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub name: Option<String>,
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of NFTs to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
    /// Sort mode
    pub sort_mode: NftSortMode,
    /// Include hidden NFTs
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub include_hidden: bool,
}

/// Response with NFTs list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftsResponse {
    /// List of NFTs
    pub nfts: Vec<NftRecord>,
    /// Total number of NFTs
    pub total: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum NftSortMode {
    Name,
    Recent,
}

/// Get a specific NFT
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Get one of the active wallet's NFTs by its NFT ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNft {
    /// NFT coin ID
    pub nft_id: String,
}

/// Response with NFT details
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftResponse {
    /// NFT if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub nft: Option<NftRecord>,
}

/// Get NFT icon image
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Get the small icon generated from an NFT's data, as base64, if it has been downloaded."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftIcon {
    /// NFT coin ID
    pub nft_id: String,
}

/// Response with NFT icon
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftIconResponse {
    /// Base64-encoded icon image
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub icon: Option<String>,
}

/// Get NFT thumbnail image
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Get the thumbnail generated from an NFT's data, as base64, if it has been downloaded."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftThumbnail {
    /// NFT coin ID
    pub nft_id: String,
}

/// Response with NFT thumbnail
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftThumbnailResponse {
    /// Base64-encoded thumbnail image
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub thumbnail: Option<String>,
}

/// Get the metadata updater of any NFT, including ones not in the wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Look up an NFT's metadata updater kind, fetching it from the network if it isn't in the wallet. Used to warn about mutable NFTs before making an offer."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftMetadataUpdater {
    /// NFT launcher ID
    pub nft_id: String,
}

/// Response with an NFT's metadata updater kind
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftMetadataUpdaterResponse {
    /// The metadata updater kind, or null if the NFT couldn't be found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub metadata_updater_kind: Option<NftUpdaterKind>,
    /// The NFT's name, if it's known
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub name: Option<String>,
}

/// Get NFT data file
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "NFTs",
        description = "Get an NFT's downloaded data and off-chain metadata, and whether each matches the hash in its on-chain metadata."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftData {
    /// NFT coin ID
    pub nft_id: String,
}

/// Response with NFT data
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "NFTs"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNftDataResponse {
    /// NFT data if available
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub data: Option<NftData>,
}

/// Check if an asset is owned
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Coins",
        description = "Check whether the active wallet holds an asset, given a CAT asset ID or an NFT, DID or option ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IsAssetOwned {
    /// Asset ID to check
    pub asset_id: String,
}

/// Response with asset ownership status
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Coins"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IsAssetOwnedResponse {
    /// Whether the asset is owned by this wallet
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub owned: bool,
}
