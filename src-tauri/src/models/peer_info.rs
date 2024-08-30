use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ip_addr: String,
    pub port: u16,
    pub trusted: bool,
}
