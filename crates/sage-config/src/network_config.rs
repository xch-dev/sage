use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NetworkConfig {
    pub network_id: String,
    pub target_peers: u32,
    pub discover_peers: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            network_id: "testnet11".to_string(),
            target_peers: 3,
            discover_peers: true,
        }
    }
}
