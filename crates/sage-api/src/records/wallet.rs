use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletRecord {
    pub name: String,
    pub fingerprint: u32,
    #[serde(flatten)]
    pub kind: WalletKind,
    pub network_id: String,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletKind {
    Bls {
        public_key: String,
        has_secrets: bool,
    },
    Vault {
        launcher_id: String,
    },
    Watch {
        addresses: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SecretKeyInfo {
    pub mnemonic: Option<String>,
    pub secret_key: String,
}
