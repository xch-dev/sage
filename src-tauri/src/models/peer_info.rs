use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PeerInfo {
    pub ip_addr: String,
    pub port: u16,
    pub trusted: bool,
}
