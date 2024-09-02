use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_network_id")]
    pub network_id: String,

    #[serde(default = "default_target_peers")]
    pub target_peers: usize,

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
    "mainnet".to_string()
}

fn default_target_peers() -> usize {
    5
}

fn default_discover_peers() -> bool {
    true
}
