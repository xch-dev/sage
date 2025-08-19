use std::{ops::Deref, sync::Arc};

use tokio::sync::{mpsc::Sender, Mutex};

use crate::{PeerState, SyncCommand};

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
    pub commands: Sender<SyncCommand>,
}

impl SyncState {
    pub fn new(commands: Sender<SyncCommand>) -> Self {
        Self(Arc::new(SyncStateInner {
            peers: Mutex::new(PeerState::default()),
            commands,
        }))
    }
}
