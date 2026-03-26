use serde::{Deserialize, Serialize};

use crate::{SecretKeyInfo, WalletRecord};

/// Login to a wallet using a fingerprint
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
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
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginResponse {}

/// Log out of the current wallet session
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
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
    crate::openapi_attr(tag = "Wallets")
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
        tag = "Wallets",
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
    crate::openapi_attr(tag = "Wallets")
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

/// Import a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Import a wallet using a mnemonic phrase or private key. Optionally saves secrets and automatically logs in."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportWallet {
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

/// Response with imported wallet fingerprint
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportWalletResponse {
    /// Fingerprint of the imported wallet
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Import a read-only wallet using a list of addresses. Optionally logs in.
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Import a read-only wallet using a list of addresses. Optionally logs in."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportAddresses {
    /// Display name for the wallet
    pub name: String,
    /// List of addresses
    pub addresses: Vec<String>,
    /// Whether to automatically login after import
    #[serde(default = "yes")]
    #[cfg_attr(feature = "openapi", schema(default = true))]
    pub login: bool,
    /// Optional emoji identifier
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub emoji: Option<String>,
}

/// Response with imported wallet fingerprint
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportAddressesResponse {
    /// Fingerprint of the imported wallet
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

/// Delete a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Permanently delete a wallet from the system. This action cannot be undone."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteWallet {
    /// Wallet fingerprint to delete
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response for wallet deletion
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeleteWalletResponse {}

/// Rename a wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Change the display name of a wallet."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RenameWallet {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// New display name
    pub name: String,
}

/// Response for wallet rename
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RenameWalletResponse {}

/// Set wallet emoji
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
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
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetWalletEmojiResponse {}

/// List all wallets
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "List all available wallets stored in the system."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWallets {}

/// Response with all wallets
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletsResponse {
    /// List of wallet records
    pub wallets: Vec<WalletRecord>,
}

/// Get a specific wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Get information about a specific wallet by fingerprint."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWallet {
    /// Wallet fingerprint (uses currently logged in if null)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true, example = 1_234_567_890))]
    pub fingerprint: Option<u32>,
}

/// Response with wallet information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletResponse {
    /// Wallet record if found
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub wallet: Option<WalletRecord>,
}

/// Get wallet secrets
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Wallets",
        description = "Retrieve the secrets (mnemonic/key) for a wallet. Requires authentication."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletSecrets {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
}

/// Response with wallet secrets
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(tag = "Wallets")
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetWalletSecretsResponse {
    /// Secret key information if authorized
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub secrets: Option<SecretKeyInfo>,
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
