use std::{
    cmp::Reverse,
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{
    protocol::{Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{connect_peer, Connector, Network, NetworkId, Peer};
use futures_lite::{future::poll_once, StreamExt};
use futures_util::stream::FuturesUnordered;
use itertools::Itertools;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{debug, info, warn};

use crate::{CatQueue, NftQueue, PuzzleQueue, Wallet, WalletError};

use super::{
    peer_event::{handle_peer, handle_peer_events, PeerEvent},
    wallet_sync::sync_wallet,
    PeerInfo, PeerState, SyncEvent,
};

#[derive(Debug, Clone, Copy)]
pub struct SyncOptions {
    pub target_peers: usize,
    pub find_peers: bool,
    pub max_peers_for_dns: usize,
    pub dns_batch_size: usize,
    pub connection_batch_size: usize,
    pub max_peer_age_seconds: u64,
    pub sync_delay: Duration,
    pub connection_timeout: Duration,
    pub initial_peak_timeout: Duration,
    pub remove_subscription_timeout: Duration,
    pub request_peers_timeout: Duration,
    pub dns_timeout: Duration,
}

pub struct SyncManager {
    options: SyncOptions,
    state: Arc<Mutex<PeerState>>,
    wallet: Option<Arc<Wallet>>,
    network_id: NetworkId,
    network: Network,
    connector: Connector,
    sync_sender: mpsc::Sender<SyncEvent>,
    peer_sender: mpsc::Sender<PeerEvent>,
    peer_receiver_task: JoinHandle<()>,
    initial_wallet_sync: InitialWalletSync,
    puzzle_lookup_task: Option<JoinHandle<Result<(), WalletError>>>,
    cat_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    nft_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
}

impl fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncManager").finish()
    }
}

enum InitialWalletSync {
    Idle,
    Syncing {
        ip: IpAddr,
        task: JoinHandle<Result<(), WalletError>>,
    },
    Subscribed(IpAddr),
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        self.peer_receiver_task.abort();
        if let InitialWalletSync::Syncing { task, .. } = &mut self.initial_wallet_sync {
            task.abort();
        }
        if let Some(task) = &mut self.puzzle_lookup_task {
            task.abort();
        }
        if let Some(task) = &mut self.cat_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.nft_queue_task {
            task.abort();
        }
    }
}

impl SyncManager {
    pub fn new(
        options: SyncOptions,
        state: Arc<Mutex<PeerState>>,
        wallet: Option<Arc<Wallet>>,
        network_id: NetworkId,
        network: Network,
        connector: Connector,
    ) -> (Self, mpsc::Receiver<SyncEvent>) {
        let (sync_sender, sync_receiver) = mpsc::channel(32);
        let (peer_sender, peer_receiver) = mpsc::channel(options.target_peers.max(1));

        let peer_receiver_task = tokio::spawn(handle_peer_events(
            wallet.as_ref().map(|wallet| wallet.db.clone()),
            peer_receiver,
            state.clone(),
            sync_sender.clone(),
        ));

        let manager = Self {
            options,
            state,
            wallet,
            network_id,
            network,
            connector,
            peer_sender,
            sync_sender,
            peer_receiver_task,
            initial_wallet_sync: InitialWalletSync::Idle,
            puzzle_lookup_task: None,
            cat_queue_task: None,
            nft_queue_task: None,
        };

        (manager, sync_receiver)
    }

    pub async fn sync(mut self) {
        self.clear_subscriptions().await;

        loop {
            self.update().await;
            sleep(self.options.sync_delay).await;
        }
    }

    async fn clear_subscriptions(&self) {
        let mut futures = FuturesUnordered::new();

        for peer in self
            .state
            .lock()
            .await
            .peers()
            .map(|info| info.peer.clone())
            .collect_vec()
        {
            let ip = peer.socket_addr().ip();

            futures.push(async move {
                let result = timeout(
                    self.options.remove_subscription_timeout,
                    peer.remove_puzzle_subscriptions(None),
                )
                .await;
                (ip, result)
            });
        }

        while let Some((ip, result)) = futures.next().await {
            match result {
                Ok(Ok(_response)) => {}
                Ok(Err(error)) => {
                    debug!("Failed to clear subscriptions from {ip}: {error}");
                }
                Err(_timeout) => {
                    debug!("Timeout clearing subscriptions from {ip}");
                }
            }
        }
    }

    async fn update(&mut self) {
        let peer_count = self.state.lock().await.peer_count();

        if self.options.find_peers && peer_count < self.options.target_peers {
            if peer_count > self.options.max_peers_for_dns {
                if !self.peer_discovery().await {
                    self.dns_discovery().await;
                }
            } else {
                self.dns_discovery().await;
            }
        }

        self.update_tasks().await;
        self.poll_tasks().await;
    }

    async fn dns_discovery(&mut self) {
        let addrs = self
            .network
            .lookup_all(self.options.dns_timeout, self.options.dns_batch_size)
            .await;

        for addrs in addrs.chunks(self.options.connection_batch_size) {
            if self.connect_batch(addrs).await {
                break;
            }
        }
    }

    async fn peer_discovery(&mut self) -> bool {
        let peers: Vec<Peer> = self
            .state
            .lock()
            .await
            .peers()
            .map(|info| info.peer.clone())
            .collect();

        if peers.is_empty() {
            warn!("No existing peers to request new peers from");
            return false;
        }

        let mut futures = FuturesUnordered::new();

        for peer in peers {
            let ip = peer.socket_addr().ip();
            let duration = self.options.request_peers_timeout;
            futures.push(async move {
                let result = timeout(duration, peer.request_peers()).await;
                (ip, result)
            });
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_secs();

        while let Some((ip, result)) = futures.next().await {
            match result {
                Ok(Ok(mut response)) => {
                    response.peer_list.retain(|item| {
                        item.timestamp >= timestamp - self.options.max_peer_age_seconds
                    });

                    if !response.peer_list.is_empty() {
                        info!(
                            "Received {} recent peers from {}",
                            response.peer_list.len(),
                            ip
                        );
                    }

                    response
                        .peer_list
                        .sort_by_key(|item| Reverse(item.timestamp));

                    let mut addrs = Vec::new();

                    for item in response.peer_list {
                        let Some(new_ip) = IpAddr::from_str(&item.host).ok() else {
                            debug!("Invalid IP address in peer list");
                            self.state.lock().await.ban(ip);
                            break;
                        };
                        addrs.push(SocketAddr::new(new_ip, self.network.default_port));
                    }

                    for addrs in addrs.chunks(self.options.connection_batch_size) {
                        if self.connect_batch(addrs).await {
                            return true;
                        }
                    }
                }
                Ok(Err(error)) => {
                    debug!("Failed to request peers from {}: {}", ip, error);
                    self.state.lock().await.ban(ip);
                }
                Err(_timeout) => {
                    debug!("Request for peers from {} timed out", ip);
                    self.state.lock().await.ban(ip);
                }
            }
        }

        false
    }

    async fn connect_batch(&mut self, addrs: &[SocketAddr]) -> bool {
        let mut futures = FuturesUnordered::new();

        for &socket_addr in addrs {
            let state = self.state.lock().await;
            if state.is_connected(socket_addr.ip()) || state.is_banned(socket_addr.ip()) {
                continue;
            }
            drop(state);

            let network_id = self.network_id.clone();
            let connector = self.connector.clone();
            let duration = self.options.connection_timeout;

            futures.push(async move {
                let result =
                    timeout(duration, connect_peer(network_id, connector, socket_addr)).await;
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
        self.state.lock().await.peer_count() >= self.options.target_peers
    }

    async fn try_add_peer(&mut self, peer: Peer, mut receiver: mpsc::Receiver<Message>) -> bool {
        let Ok(Some(message)) = timeout(self.options.initial_peak_timeout, receiver.recv()).await
        else {
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

        self.state.lock().await.add_peer(PeerInfo {
            peer,
            claimed_peak: message.height,
            header_hash: message.header_hash,
            receive_message_task: tokio::spawn(handle_peer(ip, receiver, self.peer_sender.clone())),
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
                            wallet.db.clone(),
                            wallet.intermediate_pk,
                            self.network.genesis_challenge,
                            peer,
                            self.state.clone(),
                            self.sync_sender.clone(),
                        ));
                        *sync = InitialWalletSync::Syncing { ip, task };
                        self.sync_sender.send(SyncEvent::Start(ip)).await.ok();
                    }
                }
            }
            InitialWalletSync::Syncing { ip, task }
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                task.abort();
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.sync_sender.send(SyncEvent::Stop).await.ok();
            }
            InitialWalletSync::Subscribed(ip)
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.sync_sender.send(SyncEvent::Stop).await.ok();
            }
            _ => {}
        }

        if let Some(wallet) = self.wallet.clone() {
            if self.puzzle_lookup_task.is_none() {
                let task = tokio::spawn(
                    PuzzleQueue::new(
                        wallet.db.clone(),
                        wallet.genesis_challenge,
                        self.state.clone(),
                        self.sync_sender.clone(),
                    )
                    .start(),
                );
                self.puzzle_lookup_task = Some(task);
            }

            if self.cat_queue_task.is_none() {
                let task = tokio::spawn(
                    CatQueue::new(wallet.db.clone(), self.sync_sender.clone()).start(),
                );
                self.cat_queue_task = Some(task);
            }

            if self.nft_queue_task.is_none() {
                let task = tokio::spawn(
                    NftQueue::new(wallet.db.clone(), self.sync_sender.clone()).start(),
                );
                self.nft_queue_task = Some(task);
            }
        } else {
            self.puzzle_lookup_task = None;
            self.cat_queue_task = None;
            self.nft_queue_task = None;
        }
    }

    async fn poll_tasks(&mut self) {
        if let InitialWalletSync::Syncing { ip, task } = &mut self.initial_wallet_sync {
            if let Ok(Some(result)) = timeout(Duration::from_secs(1), poll_once(task)).await {
                match result {
                    Ok(Ok(())) => {
                        self.initial_wallet_sync = InitialWalletSync::Subscribed(*ip);
                        self.sync_sender.send(SyncEvent::Subscribed).await.ok();
                    }
                    Ok(Err(error)) => {
                        warn!("Initial wallet sync failed: {error}");
                        self.state.lock().await.ban(*ip);
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.sync_sender.send(SyncEvent::Stop).await.ok();
                    }
                    Err(_timeout) => {
                        warn!("Initial wallet sync timed out");
                        self.state.lock().await.ban(*ip);
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.sync_sender.send(SyncEvent::Stop).await.ok();
                    }
                }
            }
        }

        if let Some(task) = &mut self.cat_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("CAT lookup queue failed with panic: {error}");
                    self.cat_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("CAT lookup queue failed with error: {error}");
                    self.cat_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.cat_queue_task = None;
                }
                None => {}
            }
        }

        if let Some(task) = &mut self.nft_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("NFT update queue failed with panic: {error}");
                    self.nft_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("NFT update queue failed with error: {error}");
                    self.nft_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.nft_queue_task = None;
                }
                None => {}
            }
        }
    }
}
