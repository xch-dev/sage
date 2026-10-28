use serde::{Deserialize, Serialize};

/// Public information about a key in the keychain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct KeyInfo {
    pub fingerprint: u32,
    /// The hex encoded BLS master public key
    pub public_key: String,
    /// Whether the keychain holds the secret key, rather than only the public key
    pub has_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SecretKeyInfo {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
