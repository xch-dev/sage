use tokio::task::JoinHandle;

use crate::error::Result;

#[derive(Debug)]
pub enum SyncStatus {
    Idle,
    Syncing(JoinHandle<Result<()>>),
    Synced,
}
