use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInfo {
    pub total_coins: u32,
    pub synced_coins: u32,
}
