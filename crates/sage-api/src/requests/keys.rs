use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{KeyInfo, SecretKeyInfo};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Login {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct LoginResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Logout {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct LogoutResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Resync {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ResyncResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportKey {
    pub name: String,
    pub key: String,
    #[serde(default = "yes")]
    pub save_secrets: bool,
    #[serde(default = "yes")]
    pub login: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ImportKeyResponse {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct DeleteKey {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct DeleteKeyResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetKeys {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetKeysResponse {
    pub keys: Vec<KeyInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetKey {
    #[serde(default)]
    pub fingerprint: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetKeyResponse {
    pub key: Option<KeyInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetSecretKey {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetSecretKeyResponse {
    pub secrets: Option<SecretKeyInfo>,
}
