use serde::{Deserialize, Serialize};

use super::PeerMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_network_id")]
    pub network_id: String,

    #[serde(default = "default_target_peers")]
    pub target_peers: usize,

    #[serde(default)]
    pub peer_mode: PeerMode,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_id: default_network_id(),
            target_peers: default_target_peers(),
            peer_mode: PeerMode::default(),
        }
    }
}

fn default_network_id() -> String {
    "mainnet".to_string()
}

fn default_target_peers() -> usize {
    5
}
