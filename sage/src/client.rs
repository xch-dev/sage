use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

mod event;
mod handler;
mod peer;
mod request;
mod requests;
mod response;

pub use event::*;
pub use peer::*;
pub use request::*;
pub use response::*;

#[derive(Debug, Clone, Copy)]
pub struct ClientOptions {
    target_peers: usize,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self { target_peers: 3 }
    }
}

#[derive(Debug, Clone)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
    peers: Arc<Mutex<HashMap<PeerId, Peer>>>,
    options: ClientOptions,
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self(Arc::new(ClientInner {
            peers: Arc::new(Mutex::new(HashMap::new())),
            options,
        }))
    }

    pub async fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.0.peers.lock().await.get(peer_id).cloned()
    }

    pub async fn remove_peer(&self, peer_id: &PeerId) {
        self.0.peers.lock().await.remove(peer_id);
    }
}
