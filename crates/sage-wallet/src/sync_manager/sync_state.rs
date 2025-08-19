use std::{ops::Deref, sync::Arc};

use tokio::sync::{mpsc::Sender, Mutex};

use crate::{PeerState, SyncCommand, SyncEvent};

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
    pub events: Sender<SyncEvent>,
}

impl SyncState {
    pub fn new(commands: Sender<SyncCommand>, events: Sender<SyncEvent>) -> Self {
        Self(Arc::new(SyncStateInner {
            peers: Mutex::new(PeerState::default()),
            commands,
            events,
        }))
    }
}
