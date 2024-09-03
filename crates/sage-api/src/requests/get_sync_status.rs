use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Amount;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetSyncStatus {
    pub fingerprint: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncStatus {
    pub balance: Amount,
    pub synced_coins: u32,
    pub total_coins: u32,
}
