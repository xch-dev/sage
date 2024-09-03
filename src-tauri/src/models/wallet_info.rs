use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum WalletKind {
    Cold,
    Hot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletInfo {
    pub name: String,
    pub fingerprint: u32,
    pub public_key: String,
    pub kind: WalletKind,
}
