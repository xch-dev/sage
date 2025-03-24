use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub global: GlobalConfig,
    pub network: NetworkConfig,
    pub rpc: RpcConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GlobalConfig {
    pub log_level: String,
    pub fingerprint: Option<u32>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            fingerprint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NetworkConfig {
    pub default_network: String,
    pub target_peers: u32,
    pub discover_peers: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_network: "mainnet".to_string(),
            target_peers: 5,
            discover_peers: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct RpcConfig {
    pub enabled: bool,
    pub port: u16,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 9257,
        }
    }
}
