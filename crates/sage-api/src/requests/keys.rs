use serde::{Deserialize, Serialize};

use crate::{KeyInfo, SecretKeyInfo};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct Login {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct LoginResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct Logout {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct LogoutResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[allow(clippy::struct_excessive_bools)]
pub struct Resync {
    pub fingerprint: u32,
    #[serde(default)]
    pub delete_offer_files: bool,
    #[serde(default)]
    pub delete_unhardened_derivations: bool,
    #[serde(default)]
    pub delete_hardened_derivations: bool,
    #[serde(default)]
    pub delete_blockinfo: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ResyncResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GenerateMnemonic {
    pub use_24_words: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GenerateMnemonicResponse {
    pub mnemonic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportKey {
    pub name: String,
    pub key: String,
    #[serde(default)]
    pub derivation_index: u32,
    #[serde(default = "yes")]
    pub save_secrets: bool,
    #[serde(default = "yes")]
    pub login: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct ImportKeyResponse {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteKey {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct DeleteKeyResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct RenameKey {
    pub fingerprint: u32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct RenameKeyResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetKeys {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetKeysResponse {
    pub keys: Vec<KeyInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetKey {
    #[serde(default)]
    pub fingerprint: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetKeyResponse {
    pub key: Option<KeyInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetSecretKey {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetSecretKeyResponse {
    pub secrets: Option<SecretKeyInfo>,
}
