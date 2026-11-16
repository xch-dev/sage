use serde::{Deserialize, Serialize};

use crate::{Amount, CoinSpendJson, TransactionResponse, TransactionSummary};

/// Start a recovery of the open vault wallet's vault with its recovery key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Start a recovery of the open vault with its recovery key, which moves the vault to a new custody key once the recovery timelock expires. Until then, the current custody can cancel it. The recovery key is the vault's attached recovery signer, or a mnemonic, which is stored in the keychain and attached. The new custody key is a keychain key, a mnemonic, which is stored in the keychain, or a key derived from a BLS wallet's key. The recovery can't pay a fee itself, so a fee is paid by the BLS wallet `fee_fingerprint`, if any. Refused while the vault has a pending transaction. Without `auto_submit`, nothing is stored, and the response only shows what the recovery would do."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StartVaultRecovery {
    /// The recovery key's mnemonic, if it isn't attached to the vault wallet already
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery_mnemonic: Option<String>,
    /// The new custody key
    pub custody: VaultKeySource,
    /// Transaction fee, paid by `fee_fingerprint`
    #[serde(default = "zero")]
    pub fee: Amount,
    /// The fingerprint of a BLS wallet that pays the fee
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub fee_fingerprint: Option<u32>,
    /// Whether to sign and submit the recovery
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Where a new vault key comes from
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum VaultKeySource {
    /// A keychain key with its secret key
    Keychain { fingerprint: u32 },
    /// A mnemonic, such as one from `generate_mnemonic`, which is stored in the keychain
    Mnemonic { mnemonic: String },
    /// A vault key derived from a BLS wallet's key, so that the wallet's mnemonic also backs it up
    Derived { fingerprint: u32 },
}

/// Response with a recovery transaction
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StartVaultRecoveryResponse {
    /// The fingerprint of the new custody key
    pub custody_fingerprint: u32,
    /// How long the recovery takes before it can be finished, in seconds
    pub recovery_timelock: u64,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

/// Cancel the open vault wallet's pending recovery
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Cancel the open vault's pending recovery with its current custody key, which recreates the vault as it was before the recovery started. The fee is paid from the vault's XCH. Refused while the vault has a pending transaction."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CancelVaultRecovery {
    /// Transaction fee
    #[serde(default = "zero")]
    pub fee: Amount,
    /// Whether to sign and submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Finish the open vault wallet's pending recovery
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Finish the open vault's pending recovery once its timelock has expired, by block timestamps, which moves the vault to its new custody. Finishing needs no signature, so Sage only finishes a recovery whose new custody has a key in its keychain, so that it can't hand the vault to someone else. A recovery whose memo didn't reveal its new custody can't be finished here, only cancelled. The recovery can't pay a fee itself, so a fee is paid by the BLS wallet `fee_fingerprint`, if any."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FinishVaultRecovery {
    /// Transaction fee, paid by `fee_fingerprint`
    #[serde(default = "zero")]
    pub fee: Amount,
    /// The fingerprint of a BLS wallet that pays the fee
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub fee_fingerprint: Option<u32>,
    /// Whether to submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Rekey the open vault wallet's vault
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Recreate the open vault with a new config, signed by its current custody key. Each part that isn't given is kept: the custody key, the recovery key, and the recovery timelock. The new config always uses the latest puzzle version, so rekeying a legacy vault with nothing else changed upgrades it. The new config is revealed in a memo, but it should also be backed up with `export_vault_config`. The fee is paid from the vault's XCH. Refused while the vault has a pending transaction or recovery."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RekeyVault {
    /// The new custody key, if it changes
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub custody: Option<VaultKeySource>,
    /// The new recovery key, if it changes
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery: Option<VaultKeySource>,
    /// The new recovery timelock in seconds, if it changes. It must be at least an hour
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery_timelock: Option<u64>,
    /// Transaction fee
    #[serde(default = "zero")]
    pub fee: Amount,
    /// Whether to sign and submit the transaction
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response with a rekey transaction
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RekeyVaultResponse {
    /// The fingerprint of the new custody key, if it changed
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub custody_fingerprint: Option<u32>,
    /// The fingerprint of the new recovery key, if it changed
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery_fingerprint: Option<u32>,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}

pub type CancelVaultRecoveryResponse = TransactionResponse;
pub type FinishVaultRecoveryResponse = TransactionResponse;

fn zero() -> Amount {
    Amount::u64(0)
}
