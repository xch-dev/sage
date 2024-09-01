use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use chia::{
    protocol::{Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{connect_peer, Connector, Network, NetworkId, Peer};
use futures_lite::{future::poll_once, StreamExt};
use futures_util::stream::FuturesUnordered;
use sage_wallet::Wallet;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{debug, warn};

use crate::{
    config::{NetworkConfig, PeerMode},
    error::Result,
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
    connector: Connector,
    interval: Duration,
    sender: mpsc::Sender<PeerEvent>,
    receiver_task: JoinHandle<()>,
    initial_wallet_sync: InitialWalletSync,
    puzzle_lookup_task: Option<JoinHandle<Result<()>>>,
}

enum InitialWalletSync {
    Idle,
    Syncing {
        ip: IpAddr,
        task: JoinHandle<Result<()>>,
    },
    Subscribed(IpAddr),
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        self.receiver_task.abort();
        if let InitialWalletSync::Syncing { task, .. } = &mut self.initial_wallet_sync {
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
        connector: Connector,
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
            connector,
            interval,
            sender,
            receiver_task,
            initial_wallet_sync: InitialWalletSync::Idle,
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
        self.poll_tasks().await;
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
            let connector = self.connector.clone();

            futures.push(async move {
                let result = timeout(
                    Duration::from_secs(3),
                    connect_peer(network_id, connector, socket_addr),
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
                    debug!("Failed to connect to peer {socket_addr}: {error}");
                    self.state.lock().await.ban(socket_addr.ip());
                }
                Err(_timeout) => {
                    debug!("Connection to peer {socket_addr} timed out");
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
            debug!(
                "Timeout receiving NewPeakWallet message from peer {}",
                peer.socket_addr()
            );
            return false;
        };

        if message.msg_type != ProtocolMessageTypes::NewPeakWallet {
            debug!(
                "Received unexpected message from peer {}",
                peer.socket_addr()
            );
            return false;
        }

        let Ok(message) = NewPeakWallet::from_bytes(&message.data) else {
            debug!(
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

        match &mut self.initial_wallet_sync {
            sync @ InitialWalletSync::Idle => {
                if let Some(wallet) = self.wallet.clone() {
                    if let Some(peer) = state.acquire_peer() {
                        let ip = peer.socket_addr().ip();
                        let task = tokio::spawn(sync_wallet(
                            wallet.clone(),
                            self.network.genesis_challenge,
                            peer,
                            self.state.clone(),
                        ));
                        *sync = InitialWalletSync::Syncing { ip, task };
                    }
                }
            }
            InitialWalletSync::Syncing { ip, task }
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                task.abort();
                self.initial_wallet_sync = InitialWalletSync::Idle;
            }
            InitialWalletSync::Subscribed(ip)
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                self.initial_wallet_sync = InitialWalletSync::Idle;
            }
            _ => {}
        }

        if let Some(wallet) = self.wallet.clone() {
            if self.puzzle_lookup_task.is_none() {
                let task = tokio::spawn(lookup_puzzles(wallet, self.state.clone()));
                self.puzzle_lookup_task = Some(task);
            }
        } else {
            self.puzzle_lookup_task = None;
        }
    }

    async fn poll_tasks(&mut self) {
        if let InitialWalletSync::Syncing { ip, task } = &mut self.initial_wallet_sync {
            if let Ok(Some(result)) = timeout(Duration::from_secs(1), poll_once(task)).await {
                match result {
                    Ok(Ok(())) => {
                        self.initial_wallet_sync = InitialWalletSync::Subscribed(*ip);
                    }
                    Ok(Err(error)) => {
                        warn!("Initial wallet sync failed: {error}");
                        self.state.lock().await.ban(*ip);
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                    }
                    Err(_timeout) => {
                        warn!("Initial wallet sync timed out");
                        self.state.lock().await.ban(*ip);
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                    }
                }
            }
        }
    }
}
