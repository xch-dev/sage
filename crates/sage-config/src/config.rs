use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{AppConfig, NetworkConfig, WalletConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub wallet: GeneralWalletConfig,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub wallets: IndexMap<String, WalletConfig>,
    #[serde(default)]
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallet: GeneralWalletConfig::default(),
            wallets: IndexMap::new(),
            network: NetworkConfig::default(),
            app: AppConfig::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct GeneralWalletConfig {
    #[serde(default)]
    pub active_fingerprint: Option<u32>,
}
