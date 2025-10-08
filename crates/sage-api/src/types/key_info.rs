use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct KeyInfo {
    pub name: String,
    pub fingerprint: u32,
    pub public_key: String,
    pub kind: KeyKind,
    pub has_secrets: bool,
    pub network_id: String,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
    Bls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SecretKeyInfo {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
