use serde::{Deserialize, Serialize};

/// Public information about a key in the keychain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct KeyInfo {
    pub fingerprint: u32,
    pub kind: KeyKind,
    /// The hex encoded public key. For BLS keys, this is the master public key
    pub public_key: String,
    /// Whether the keychain holds the secret key, rather than only the public key
    pub has_secrets: bool,
}

/// The kind of a key in the keychain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
    /// A BLS key, which can be a wallet of its own
    Bls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SecretKeyInfo {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
