use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod derivation_mode;
mod network_config;
mod peer_mode;
mod wallet_config;

pub use derivation_mode::*;
pub use network_config::*;
pub use peer_mode::*;
pub use wallet_config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub wallets: IndexMap<String, WalletConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_wallet: Option<u32>,
    #[serde(default)]
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallets: IndexMap::new(),
            active_wallet: None,
            network: NetworkConfig::default(),
        }
    }
}
