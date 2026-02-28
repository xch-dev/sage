use serde::{Deserialize, Serialize};

use crate::{KeyInfo, SecretKeyInfo};

/// Login to a wallet using a fingerprint
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Authenticate and log into a wallet using its fingerprint. This must be called before most other endpoints."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Login {
    /// The unique fingerprint identifier of the wallet to authenticate with. This is a 32-bit unsigned integer that uniquely identifies each wallet key in the system.
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response from logging into a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginResponse {}

/// Log out of the current wallet session
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Log out of the current wallet session and clear authentication."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Logout {}

/// Response from logging out of a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LogoutResponse {}

/// Resynchronize wallet data with the blockchain
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System & Sync",
        description = "Resynchronize wallet data with the blockchain. Can optionally delete coins, assets, files, offers, addresses, or blocks."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[allow(clippy::struct_excessive_bools)]
pub struct Resync {
    /// The fingerprint of the wallet to resync
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Delete all coin records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_coins: bool,
    /// Delete all asset records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_assets: bool,
    /// Delete all file records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_files: bool,
    /// Delete all offer records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_offers: bool,
    /// Delete all address records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_addresses: bool,
    /// Delete all block records during resync
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = false))]
    pub delete_blocks: bool,
}

/// Response from resynchronizing the wallet
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System & Sync"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResyncResponse {}

/// Generate a new mnemonic phrase for wallet creation
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Generate a new BIP-39 mnemonic phrase (12 or 24 words) for wallet creation."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GenerateMnemonic {
    /// Whether to generate a 24-word mnemonic instead of 12-word
    #[cfg_attr(feature = "openapi", schema(example = false, default = false))]
    pub use_24_words: bool,
}

/// Response containing the generated mnemonic phrase
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GenerateMnemonicResponse {
    /// The generated BIP-39 mnemonic phrase
    #[cfg_attr(
        feature = "openapi",
        schema(
            example = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        )
    )]
    pub mnemonic: String,
}

/// Import a wallet key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Import a wallet using a mnemonic phrase or private key. Optionally saves secrets and automatically logs in."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportKey {
    /// Display name for the wallet
    pub name: String,
    /// Mnemonic phrase or private key
    pub key: String,
    /// Starting derivation index
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(default = 0))]
    pub derivation_index: u32,
    /// Optional hardened derivation count
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub hardened: Option<bool>,
    /// Optional unhardened derivation count
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub unhardened: Option<bool>,
    /// Whether to save secrets to keychain
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub save_secrets: bool,
    /// Whether to automatically login after import
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub login: bool,
    /// Optional emoji identifier
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub emoji: Option<String>,
}

fn yes() -> bool {
    true
}

/// Response with imported key fingerprint
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportKeyResponse {
    /// Fingerprint of the imported key
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Delete a wallet database
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "System & Sync",
        description = "Delete the wallet database for a specific fingerprint and network."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteDatabase {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Network name
    pub network: String,
}

/// Response for database deletion
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "System & Sync"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteDatabaseResponse {}

/// Delete a wallet key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Permanently delete a wallet key from the system. This action cannot be undone."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteKey {
    /// Wallet fingerprint to delete
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response for key deletion
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteKeyResponse {}

/// Rename a wallet key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Change the display name of a wallet key."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RenameKey {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// New display name
    pub name: String,
}

/// Response for key rename
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RenameKeyResponse {}

/// Set wallet emoji
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Set an emoji identifier/avatar for a wallet to make it easier to distinguish."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetWalletEmoji {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Emoji character (null to remove)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub emoji: Option<String>,
}

/// Response for emoji update
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetWalletEmojiResponse {}

/// List all wallet keys
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "List all available wallet keys stored in the system."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetKeys {}

/// Response with all wallet keys
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetKeysResponse {
    /// List of wallet keys
    pub keys: Vec<KeyInfo>,
}

/// Get a specific wallet key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Get information about a specific wallet key by fingerprint."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetKey {
    /// Wallet fingerprint (uses currently logged in if null)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true, example = 1_234_567_890))]
    pub fingerprint: Option<u32>,
}

/// Response with key information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetKeyResponse {
    /// Key information if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub key: Option<KeyInfo>,
}

/// Get wallet secret key
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Retrieve the secret key (mnemonic) for a wallet. Requires authentication."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSecretKey {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response with secret key information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSecretKeyResponse {
    /// Secret key information if authorized
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub secrets: Option<SecretKeyInfo>,
}

/// Get the receive address for any wallet without switching sessions
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Authentication & Keys",
        description = "Get the current receive address for any wallet by fingerprint without switching the active session."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletAddress {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response with the wallet's receive address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Authentication & Keys")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletAddressResponse {
    /// The wallet's current receive address
    pub address: String,
}

/// List all custom theme NFTs
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "List all custom theme NFTs in the wallet."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetUserThemes {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetUserThemesResponse {
    /// List of theme NFT IDs
    pub themes: Vec<String>,
}

/// Get a specific theme NFT
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "Retrieve a specific custom theme NFT by its ID."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetUserTheme {
    /// NFT ID of the theme
    pub nft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetUserThemeResponse {
    /// Theme data if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub theme: Option<String>,
}

/// Save a theme NFT to the wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "Save a custom theme NFT to the wallet for use in the UI."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveUserTheme {
    /// NFT ID of the theme
    pub nft_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SaveUserThemeResponse {}

/// Delete a theme NFT from the wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "Remove a custom theme NFT from the wallet."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteUserTheme {
    /// NFT ID of the theme
    pub nft_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteUserThemeResponse {}
