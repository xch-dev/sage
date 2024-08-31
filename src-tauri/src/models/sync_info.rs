use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInfo {
    pub xch_balance: String,
    pub total_coins: u32,
    pub synced_coins: u32,
}
