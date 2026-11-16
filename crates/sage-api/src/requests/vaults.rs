use serde::{Deserialize, Serialize};

use crate::{Amount, CoinSpendJson, TransactionSummary, VaultRecord};

/// Import an existing vault as a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Import an existing vault, minted by Sage or Chia Cloud Wallet, as a read-only wallet. A vault can't be found from a key, so it's imported from exactly one of its launcher id, an address it has already spent from, or a config file. A config file is only accepted if it rebuilds the vault's custody on chain. Otherwise, the vault's config is read from the memos on chain, if it has them. For a vault without memos, its config is rebuilt from what its spends reveal. A vault without recovery needs nothing else, but a vault with recovery needs its recovery keys, given with its launcher id or address. The recovery threshold and timelock are searched for if they aren't given. Needs a connected peer. Optionally logs in automatically."
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
    /// A vault config file, as exported by Sage or Chia Cloud Wallet
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub config: Option<String>,
    /// The vault's recovery keys, to rebuild its config from its spends if it has no memos. It can't
    /// be given with a config file
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery: Option<VaultRecoveryKeys>,
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

/// A vault's recovery keys, which identify the recovery side of a vault without memos
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct VaultRecoveryKeys {
    /// The key of every recovery member: a public key in hex (BLS, secp256r1, secp256k1, or a
    /// passkey), a BLS mnemonic or secret key, whose master public key is used, or the `vault1` id of
    /// a vault. Secrets aren't stored
    pub keys: Vec<String>,
    /// How many recovery members must sign. Every threshold is tried if it isn't given
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub threshold: Option<u32>,
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

/// Export the open vault wallet's config file
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Export the open vault's config in the format Chia Cloud Wallet uses, so that it can be imported again, by Sage or Cloud Wallet. While recovery is pending, this is the config from before it started, which can still cancel it."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExportVaultConfig {}

/// Response with the vault config file
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExportVaultConfigResponse {
    /// The config file, as JSON
    pub config: String,
}

/// Import a config file for the open vault wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Import a config file for the open vault wallet, such as for a vault without memos on chain. It's only accepted if it rebuilds the vault's current custody hash."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportVaultConfig {
    /// The config file, as JSON
    pub config: String,
}

/// Response for importing a vault config file
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportVaultConfigResponse {}

/// Rebuild the open vault wallet's config from its spends
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Rebuild the config of the open vault wallet, for a vault without memos on chain. A spend through the vault's custody reveals its custody members. That's the whole config of a vault without recovery. For a vault with recovery, its recovery keys, threshold and timelock rebuild the rest, and the config is checked against the vault's custody hash. An M of N custody is only known once each of its members has signed a spend. Needs a connected peer."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RebuildVaultConfig {
    /// The vault's recovery keys, if it has recovery
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery: Option<VaultRecoveryKeys>,
}

/// Response with the rebuilt config's recovery threshold and timelock
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RebuildVaultConfigResponse {
    /// How many recovery members must sign, which was found if it wasn't given, or none if the
    /// vault has no recovery
    pub recovery_threshold: Option<u32>,
    /// The recovery timelock in seconds, which was found if it wasn't given, or none if the vault
    /// has no recovery
    pub recovery_timelock: Option<u64>,
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

/// Mint a new vault, funded by the open BLS wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Vaults",
        description = "Mint a new vault with a BLS custody key and a BLS recovery key, in the layout Chia Cloud Wallet uses. The vault's singleton and the fee are paid by the open BLS wallet. The custody key is derived from the open wallet's key unless an existing keychain key is given, and the recovery key is a mnemonic or an existing keychain key. Both are stored in the keychain, and the new vault wallet is added with them as its signers. The recovery key exists to recover the vault if the custody key is lost, so it must also be backed up offline. Without `auto_submit`, nothing is stored, and the response only shows what the mint would do."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MintVault {
    /// Display name for the vault wallet
    pub name: String,
    /// Optional emoji identifier
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub emoji: Option<String>,
    /// The fingerprint of a keychain key with a secret key to use as the custody key. If it isn't
    /// given, a custody key is derived from the open wallet's key, so its mnemonic backs it up
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub custody_fingerprint: Option<u32>,
    /// The recovery key's mnemonic, such as one from `generate_mnemonic`
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery_mnemonic: Option<String>,
    /// The fingerprint of a keychain key with a secret key to use as the recovery key, instead of
    /// a mnemonic
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub recovery_fingerprint: Option<u32>,
    /// How long a recovery takes before it can be finished, in seconds. It must be at least an
    /// hour, so that a recovery this wallet didn't start can be cancelled in time
    #[cfg_attr(feature = "openapi", schema(example = 43_200))]
    pub recovery_timelock: u64,
    /// Transaction fee
    pub fee: Amount,
    /// Whether to sign and submit the mint, and add the vault wallet
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub auto_submit: bool,
}

/// Response with the mint transaction and the vault's launcher id
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Vaults"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MintVaultResponse {
    /// The vault's launcher id, as a `vault1` id
    pub launcher_id: String,
    /// The fingerprint of the new vault wallet, once the mint is submitted
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub fingerprint: Option<u32>,
    /// The fingerprints of the custody and recovery keys
    pub custody_fingerprint: u32,
    pub recovery_fingerprint: u32,
    /// Transaction summary
    pub summary: TransactionSummary,
    /// Coin spends in the transaction
    pub coin_spends: Vec<CoinSpendJson>,
}
