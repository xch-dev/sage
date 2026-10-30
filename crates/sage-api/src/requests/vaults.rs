use serde::{Deserialize, Serialize};

use crate::VaultRecord;

/// Import an existing vault as a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Import an existing vault, minted by Sage or Chia Cloud Wallet, as a read-only wallet. A vault can't be found from a key, so it's imported from exactly one of its launcher id or an address it has already spent from. Its config is read from the memos on chain, if it has them. For a vault without memos, give its recovery key, and its config is rebuilt from what its spends reveal. The recovery timelock is searched for among common values if it isn't given. Needs a connected peer. Optionally logs in automatically."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportVault {
    /// Display name for the wallet
    pub name: String,
    /// The vault's launcher id, as a `vault1` id or in hex
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub launcher_id: Option<String>,
    /// An address the vault has spent from
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub address: Option<String>,
    /// The vault's recovery key, to rebuild its config from its spends if it has no memos
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery: Option<VaultRecoveryKey>,
    /// Whether to automatically login after import
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub login: bool,
    /// Optional emoji identifier
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub emoji: Option<String>,
}

/// Response with the imported vault wallet's fingerprint
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportVaultResponse {
    /// Fingerprint assigned to the wallet. It's derived from the launcher id, since a vault has no
    /// key of its own
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// The vault's launcher id, as a `vault1` id
    pub launcher_id: String,
}

/// A vault's recovery key, which identifies the recovery side of a vault without memos
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultRecoveryKey {
    /// The recovery member's public key in hex (BLS, secp256r1, secp256k1, or a passkey), or a
    /// BLS mnemonic or secret key, whose master public key is used. The secret isn't stored
    pub key: String,
    /// The recovery timelock in seconds. Common timelocks are tried if it isn't given
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub timelock: Option<u64>,
}

/// Get the open vault wallet's vault
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Get the open vault wallet's state, its config if it's known, and the keys that sign for it."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetVault {}

/// Response with the open vault
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetVaultResponse {
    pub vault: VaultRecord,
}

/// Attach a key that signs for a vault
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Add a BLS key to the keychain as a signer of a vault wallet. The key must be a custody or recovery member of the vault's config, which must be known. A custody signer will be able to spend the vault, and a recovery signer to start recovery."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttachVaultSigner {
    /// Fingerprint of the vault wallet
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// The signer's mnemonic, or its secret key in hex
    pub key: String,
}

/// Response with the attached signer
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttachVaultSignerResponse {
    /// Fingerprint of the signer's key in the keychain
    pub fingerprint: u32,
    /// Whether the key is a custody member
    pub custody: bool,
    /// Whether the key is a recovery member
    pub recovery: bool,
}

/// Rebuild the open vault wallet's config from its spends and its recovery key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Rebuild the config of the open vault wallet, for a vault without memos on chain. A spend through the vault's custody reveals its custody members, and the recovery key and timelock rebuild the rest, which is checked against the vault's custody hash. An M of N custody is only known once each of its members has signed a spend. Needs a connected peer."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RebuildVaultConfig {
    pub recovery: VaultRecoveryKey,
}

/// Response with the rebuilt config's recovery timelock
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RebuildVaultConfigResponse {
    /// The recovery timelock in seconds, which was found if it wasn't given
    pub recovery_timelock: u64,
}

/// Find the vault that owns an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Find the vault that owns an address, from the puzzle of a coin it has spent from the address. Returns no launcher id if nothing has been spent from the address yet, or it wasn't spent by a vault. Needs a connected peer."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FindVault {
    pub address: String,
}

/// Response with the vault that owns the address, if one was found
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FindVaultResponse {
    /// The vault's launcher id, as a `vault1` id
    pub launcher_id: Option<String>,
}

fn yes() -> bool {
    true
}
