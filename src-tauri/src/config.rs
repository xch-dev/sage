use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub wallets: IndexMap<String, WalletConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_wallet: Option<u32>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationMode {
    #[default]
    Automatic,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub name: String,
    #[serde(default)]
    pub derivation_mode: DerivationMode,
    #[serde(default = "default_derivation_batch_size")]
    pub derivation_batch_size: u32,
    #[serde(default)]
    pub derivation_index: u32,
}

fn default_derivation_batch_size() -> u32 {
    500
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Wallet".to_string(),
            derivation_mode: DerivationMode::default(),
            derivation_batch_size: default_derivation_batch_size(),
            derivation_index: 0,
        }
    }
}
