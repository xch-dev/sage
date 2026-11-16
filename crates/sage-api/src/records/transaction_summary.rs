use serde::{Deserialize, Serialize};

use crate::{Amount, Asset, HandleRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionSummary {
    /// The fee the transaction actually pays: the value of the coins it spends minus the
    /// value of the coins it creates. If that's less than the declared fee, so the
    /// transaction isn't complete on its own, this is the declared fee instead. It's only the fee paid
    /// if the transaction is submitted as is: when only signatures are returned, spends that
    /// don't need them could be left out.
    pub fee: Amount,
    /// The sum of the transaction's `RESERVE_FEE` conditions. When this differs from `fee`,
    /// some of the fee isn't declared.
    #[serde(default = "zero_amount")]
    pub declared_fee: Amount,
    pub inputs: Vec<TransactionInput>,
    /// The handle recipients of the transaction, and the verified addresses they were
    /// resolved to
    #[serde(default)]
    pub handles: Vec<HandleRecord>,
    /// What the open vault's spend in the transaction does, checked with clear signing, if the
    /// open wallet is a vault and the transaction spends it
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub vault: Option<VaultSpendRecord>,
}

/// What a vault spend does, checked with clear signing before the vault's custody signs it
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultSpendRecord {
    /// The vault's launcher id, as a vault bech32m id
    pub launcher_id: String,
    /// Whether clear signing could verify what the spend does. If not, `error` says why, and the
    /// vault won't sign it
    pub verified: bool,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub error: Option<String>,
    /// Whether the spend recreates the vault with a different custody, which Sage refuses to sign
    pub custody_changed: bool,
    /// Whether the spend melts the vault, which Sage refuses to sign
    pub melted: bool,
    /// The fee that the vault and the spends it approves reserve
    pub reserved_fee: Amount,
    /// XCH that leaves the vault's spends without being accounted for, which would be left as
    /// an extra fee
    pub unaccounted_xch: Amount,
}

fn zero_amount() -> Amount {
    Amount::u64(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SpendBundleJson {
    pub coin_spends: Vec<CoinSpendJson>,
    pub aggregated_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CoinSpendJson {
    pub coin: CoinJson,
    pub puzzle_reveal: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CoinJson {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionInput {
    pub coin_id: String,
    pub amount: Amount,
    pub address: String,
    pub asset: Option<Asset>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TransactionOutput {
    pub coin_id: String,
    pub amount: Amount,
    pub address: String,
    pub receiving: bool,
    pub burning: bool,
}
