use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    #[serde_as(as = "Hex")]
    pub node_id: [u8; 32],
    pub ip_addr: String,
    pub port: u16,
    pub trusted: bool,
}
