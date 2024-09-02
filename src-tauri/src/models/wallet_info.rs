use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletKind {
    Cold,
    Hot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub name: String,
    pub fingerprint: u32,
    pub public_key: String,
    pub kind: WalletKind,
}
