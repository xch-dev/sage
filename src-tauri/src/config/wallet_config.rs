use serde::{Deserialize, Serialize};

use super::DerivationMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default = "default_wallet_name")]
    pub name: String,
    #[serde(default)]
    pub derivation_mode: DerivationMode,
    #[serde(default = "default_derivation_batch_size")]
    pub derivation_batch_size: u32,
    #[serde(default)]
    pub derivation_index: u32,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: default_wallet_name(),
            derivation_mode: DerivationMode::default(),
            derivation_batch_size: default_derivation_batch_size(),
            derivation_index: 0,
        }
    }
}

fn default_wallet_name() -> String {
    "Unnamed Wallet".to_string()
}

fn default_derivation_batch_size() -> u32 {
    500
}
