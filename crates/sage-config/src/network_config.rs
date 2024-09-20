use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NetworkConfig {
    #[serde(default = "default_network_id")]
    pub network_id: String,

    #[serde(default = "default_target_peers")]
    pub target_peers: u32,

    #[serde(default = "default_discover_peers")]
    pub discover_peers: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_id: default_network_id(),
            target_peers: default_target_peers(),
            discover_peers: default_discover_peers(),
        }
    }
}

fn default_network_id() -> String {
    "testnet11".to_string()
}

fn default_target_peers() -> u32 {
    3
}

fn default_discover_peers() -> bool {
    true
}
