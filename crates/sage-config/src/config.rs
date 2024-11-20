use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{AppConfig, NetworkConfig, RpcConfig, WalletConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub app: AppConfig,
    pub rpc: RpcConfig,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub wallets: IndexMap<String, WalletConfig>,
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            rpc: RpcConfig::default(),
            wallets: IndexMap::new(),
            network: NetworkConfig::default(),
        }
    }
}
