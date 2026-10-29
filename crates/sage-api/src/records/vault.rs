use serde::{Deserialize, Serialize};

/// A vault, along with its current config and the keys that sign for it
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultRecord {
    /// The vault's launcher id, in hex
    pub launcher_id: String,
    pub state: VaultState,
    /// When recovery was started, in unix seconds, while it's pending and the block's timestamp
    /// is known
    pub recovery_started_at: Option<u64>,
    /// The id of the vault's latest singleton coin, once it's been synced
    pub coin_id: Option<String>,
    /// The inner puzzle hash of the vault's latest coin, if it's known
    pub custody_hash: Option<String>,
    /// The config which rebuilds the custody hash, if it's known. Without it, the vault's balances
    /// are still shown, but its signers can't be checked
    pub config: Option<VaultConfigRecord>,
    /// The keychain keys that sign for the vault
    pub signers: Vec<VaultSignerRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum VaultState {
    /// None of the vault's coins have been synced yet
    Unsynced,
    Active,
    /// Recovery has started, and the vault moves to its recovery custody once the timelock
    /// expires, unless its current custody cancels it first
    RecoveryPending,
    Melted,
}

/// A vault's layout, which rebuilds its custody hash
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultConfigRecord {
    pub version: VaultPuzzleVersion,
    pub source: VaultConfigSource,
    /// The members which can spend the vault
    pub custody: VaultSignersRecord,
    /// The members which can start recovery. While recovery is pending, these are the members of
    /// the config from before it started
    pub recovery: Option<VaultSignersRecord>,
    /// How long recovery takes to finish, in seconds
    pub recovery_timelock: u64,
    /// The custody hash which the vault moves to once recovery finishes, while it's pending
    pub recovery_custody_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum VaultPuzzleVersion {
    Legacy,
    ForceSingletonRecreation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum VaultConfigSource {
    /// Decoded from a memo on chain
    Memo,
    /// Imported from a config file
    ConfigFile,
    Unknown,
}

/// The members of a vault's custody or recovery, of which `threshold` must sign
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultSignersRecord {
    pub threshold: u32,
    pub members: Vec<VaultMemberRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultMemberRecord {
    pub kind: VaultMemberKind,
    /// The member's public key, or the launcher id of a vault member, in hex
    pub key: String,
    pub fast_forward: bool,
    /// The fingerprint of the keychain key that signs as this member, if one is attached
    pub signer_fingerprint: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum VaultMemberKind {
    Bls,
    #[serde(rename = "secp256r1")]
    Secp256r1,
    #[serde(rename = "secp256k1")]
    Secp256k1,
    Passkey,
    /// Another vault
    Vault,
    /// A member whose puzzle isn't known, only its hash
    Custom,
}

/// A keychain key which signs for a vault
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultSignerRecord {
    pub fingerprint: u32,
    /// Whether the key is a custody member, which can spend the vault
    pub custody: bool,
    /// Whether the key is a recovery member, which can start recovery
    pub recovery: bool,
}
