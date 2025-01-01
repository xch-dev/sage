use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PeerRecord {
    pub ip_addr: String,
    pub port: u16,
    pub peak_height: u32,
}
