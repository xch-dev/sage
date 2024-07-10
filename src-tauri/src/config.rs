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
    Generate,
    Cycle,
    Reuse,
}

impl DerivationMode {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Generate)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub name: String,

    #[serde(default, skip_serializing_if = "DerivationMode::is_default")]
    pub derivation_mode: DerivationMode,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Wallet".to_string(),
            derivation_mode: DerivationMode::Generate,
        }
    }
}
