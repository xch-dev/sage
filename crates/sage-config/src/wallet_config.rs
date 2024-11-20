use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WalletConfig {
    pub name: String,
    pub derive_automatically: bool,
    pub derivation_batch_size: u32,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Wallet".to_string(),
            derive_automatically: true,
            derivation_batch_size: 500,
        }
    }
}
