use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

mod event;
mod peer;
mod request;
mod requests;
mod response;

pub use event::*;
pub use peer::*;
pub use request::*;
pub use response::*;

#[derive(Debug, Default, Clone)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug, Default)]
struct ClientInner {
    peers: Mutex<HashMap<PeerId, Peer>>,
}

impl Client {
    pub fn new() -> Self {
        Self(Arc::new(ClientInner {
            peers: Mutex::new(HashMap::new()),
        }))
    }

    pub async fn peer(&self, peer_id: &PeerId) -> Option<Peer> {
        self.0.peers.lock().await.get(peer_id).cloned()
    }

    pub async fn remove_peer(&self, peer_id: &PeerId) {
        self.0.peers.lock().await.remove(peer_id);
    }
}
