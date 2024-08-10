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
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GeneralWalletConfig {
    #[serde(default)]
    pub active_fingerprint: Option<u32>,
}
