use std::{ops::Deref, sync::Arc};

use tokio::sync::Mutex;

use crate::PeerState;

#[derive(Debug, Clone)]
pub struct SyncState(Arc<SyncStateInner>);

impl Deref for SyncState {
    type Target = SyncStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct SyncStateInner {
    pub peers: Mutex<PeerState>,
}

impl SyncState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Arc::new(SyncStateInner {
            peers: Mutex::new(PeerState::default()),
        }))
    }
}
