use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use chia::{
    protocol::{Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{connect_peer, Network, NetworkId, Peer};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use native_tls::TlsConnector;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::info;

use crate::{
    config::{NetworkConfig, PeerMode},
    wallet::Wallet,
};

use super::{
    peer_state::{handle_peer, handle_peer_events, PeerEvent, PeerState},
    sync_state::SyncState,
    wallet_sync::{lookup_puzzles, sync_wallet},
};

pub struct SyncManager {
    state: Arc<Mutex<SyncState>>,
    wallet: Option<Arc<Wallet>>,
    network_id: NetworkId,
    network: Network,
    network_config: NetworkConfig,
    tls_connector: TlsConnector,
    interval: Duration,
    sender: mpsc::Sender<PeerEvent>,
    receiver_task: JoinHandle<()>,
    wallet_sync_task: Option<(IpAddr, JoinHandle<()>)>,
    puzzle_lookup_task: Option<JoinHandle<()>>,
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        self.receiver_task.abort();
        if let Some((_, task)) = &mut self.wallet_sync_task {
            task.abort();
        }
        if let Some(task) = &mut self.puzzle_lookup_task {
            task.abort();
        }
    }
}

impl SyncManager {
    pub fn new(
        state: Arc<Mutex<SyncState>>,
        wallet: Option<Arc<Wallet>>,
        network_id: NetworkId,
        network: Network,
        network_config: NetworkConfig,
        tls_connector: TlsConnector,
        interval: Duration,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(network_config.target_peers.max(1));
        let receiver_task = tokio::spawn(handle_peer_events(receiver, state.clone()));

        Self {
            state,
            wallet,
            network_id,
            network,
            network_config,
            tls_connector,
            interval,
            sender,
            receiver_task,
            wallet_sync_task: None,
            puzzle_lookup_task: None,
        }
    }

    pub async fn sync(mut self) {
        loop {
            self.update().await;
            sleep(self.interval).await;
        }
    }

    async fn update(&mut self) {
        let peer_count = self.state.lock().await.peer_count();

        if self.network_config.peer_mode == PeerMode::Automatic
            && peer_count < self.network_config.target_peers
        {
            self.dns_discovery().await;
        }

        self.update_tasks().await;
    }

    async fn dns_discovery(&mut self) {
        let addrs = self.network.lookup_all(Duration::from_secs(3), 10).await;

        for addrs in addrs.chunks(10) {
            if self.connect_batch(addrs).await {
                break;
            }
        }
    }

    async fn connect_batch(&mut self, addrs: &[SocketAddr]) -> bool {
        let mut futures = FuturesUnordered::new();

        for &socket_addr in addrs {
            if self.state.lock().await.is_banned(socket_addr.ip()) {
                continue;
            }

            let network_id = self.network_id.clone();
            let tls_connector = self.tls_connector.clone();

            futures.push(async move {
                let result = timeout(
                    Duration::from_secs(3),
                    connect_peer(network_id, tls_connector, socket_addr),
                )
                .await;
                (socket_addr, result)
            });
        }

        while let Some((socket_addr, result)) = futures.next().await {
            match result {
                Ok(Ok((peer, receiver))) => {
                    if self.try_add_peer(peer, receiver).await {
                        if self.check_peer_count().await {
                            return true;
                        }
                    } else {
                        self.state.lock().await.ban(socket_addr.ip());
                    }
                }
                Ok(Err(error)) => {
                    info!("Failed to connect to peer {socket_addr}: {error}");
                    self.state.lock().await.ban(socket_addr.ip());
                }
                Err(_timeout) => {
                    info!("Connection to peer {socket_addr} timed out");
                    self.state.lock().await.ban(socket_addr.ip());
                }
            }
        }

        self.check_peer_count().await
    }

    async fn check_peer_count(&mut self) -> bool {
        self.state.lock().await.peer_count() >= self.network_config.target_peers
    }

    async fn try_add_peer(&mut self, peer: Peer, mut receiver: mpsc::Receiver<Message>) -> bool {
        let Ok(Some(message)) = timeout(Duration::from_secs(2), receiver.recv()).await else {
            info!(
                "Timeout receiving NewPeakWallet message from peer {}",
                peer.socket_addr()
            );
            return false;
        };

        if message.msg_type != ProtocolMessageTypes::NewPeakWallet {
            info!(
                "Received unexpected message from peer {}",
                peer.socket_addr()
            );
            return false;
        }

        let Ok(message) = NewPeakWallet::from_bytes(&message.data) else {
            info!(
                "Invalid NewPeakWallet message from peer {}",
                peer.socket_addr()
            );
            return false;
        };

        let ip = peer.socket_addr().ip();

        self.state.lock().await.add_peer(PeerState {
            peer,
            claimed_peak: message.height,
            header_hash: message.header_hash,
            task: tokio::spawn(handle_peer(ip, receiver, self.sender.clone())),
        });

        true
    }

    async fn update_tasks(&mut self) {
        let state = self.state.lock().await;

        if let Some(task) = &mut self.wallet_sync_task {
            if !state.is_connected(task.0) {
                task.1.abort();
                self.wallet_sync_task = None;
            }
        }

        if let Some(wallet) = self.wallet.clone() {
            if self.wallet_sync_task.is_none() {
                if let Some(peer) = state.acquire_peer() {
                    let ip = peer.socket_addr().ip();
                    let task = tokio::spawn(sync_wallet(wallet.clone(), peer));
                    self.wallet_sync_task = Some((ip, task));
                }
            }

            if self.puzzle_lookup_task.is_none() {
                let task = tokio::spawn(lookup_puzzles(wallet, self.state.clone()));
                self.puzzle_lookup_task = Some(task);
            }
        } else {
            self.wallet_sync_task = None;
            self.puzzle_lookup_task = None;
        }
    }
}
