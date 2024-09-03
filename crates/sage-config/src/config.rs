use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{AppConfig, NetworkConfig, WalletConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub wallets: IndexMap<String, WalletConfig>,

    #[serde(default)]
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallets: IndexMap::new(),
            network: NetworkConfig::default(),
            app: AppConfig::default(),
        }
    }
}
