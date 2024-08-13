use std::{collections::HashMap, net::SocketAddr, sync::Arc};

mod initial_sync;
mod sync_status;
mod wallet_peer;

use initial_sync::initial_sync;
use sage_client::Peer;
use tauri::{AppHandle, Emitter};

pub use sync_status::*;
pub use wallet_peer::*;

use crate::wallet::Wallet;

#[derive(Debug)]
pub struct SyncManager {
    app_handle: AppHandle,
    wallet_peers: HashMap<SocketAddr, WalletPeer>,
    wallet: Option<Arc<Wallet>>,
}

impl SyncManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            wallet_peers: HashMap::new(),
            wallet: None,
        }
    }

    pub fn len(&self) -> usize {
        self.wallet_peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.wallet_peers.is_empty()
    }

    pub fn peers(&self) -> impl Iterator<Item = &Peer> {
        self.wallet_peers.values().map(|peer| &peer.peer)
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.wallet_peers.insert(
            peer.socket_addr(),
            WalletPeer {
                peer: peer.clone(),
                sync_status: self.spawn_sync(peer),
            },
        );
        self.peer_update();
    }

    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        if let Some(peer) = self.wallet_peers.remove(addr) {
            if let SyncStatus::Syncing(task) = peer.sync_status {
                task.abort();
            }
        }
        self.peer_update();
    }

    pub fn switch_wallet(&mut self, wallet: Option<Arc<Wallet>>) {
        self.wallet = wallet;
        for socket_addr in self.wallet_peers.keys().copied().collect::<Vec<_>>() {
            let sync_status = self.spawn_sync(self.wallet_peers[&socket_addr].peer.clone());
            self.wallet_peers.get_mut(&socket_addr).unwrap().sync_status = sync_status;
        }
        self.peer_update();
    }

    fn spawn_sync(&self, peer: Peer) -> SyncStatus {
        if let Some(wallet) = &self.wallet {
            SyncStatus::Syncing(tokio::spawn(initial_sync(peer, wallet.clone())))
        } else {
            SyncStatus::Idle
        }
    }

    fn peer_update(&self) {
        self.app_handle.emit("peer-update", ()).unwrap();
    }
}
