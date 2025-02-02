use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct KeyInfo {
    pub name: String,
    pub fingerprint: u32,
    pub public_key: String,
    pub kind: KeyKind,
    pub has_secrets: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
    Bls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SecretKeyInfo {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
