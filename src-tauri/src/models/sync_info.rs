use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncInfo {
    pub address: String,
    pub balance: String,
    pub ticker: String,
    pub total_coins: u32,
    pub synced_coins: u32,
}
