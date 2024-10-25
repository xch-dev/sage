use std::{
    fmt, mem,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use chia::{
    protocol::{Bytes32, CoinStateUpdate, Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{ClientError, Connector, Network};
use futures_lite::future::poll_once;
use itertools::Itertools;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{debug, info, warn};
use wallet_sync::{incremental_sync, sync_wallet};

use crate::{NftQueue, PuzzleQueue, TransactionQueue, Wallet, WalletError};

mod options;
mod peer_discovery;
mod peer_state;
mod sync_command;
mod sync_event;
mod wallet_sync;

pub use options::*;
pub use peer_state::*;
pub use sync_command::*;
pub use sync_event::*;

pub struct SyncManager {
    options: SyncOptions,
    state: Arc<Mutex<PeerState>>,
    wallet: Option<Arc<Wallet>>,
    network_id: String,
    network: Network,
    connector: Connector,
    event_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
    command_receiver: mpsc::Receiver<SyncCommand>,
    initial_wallet_sync: InitialWalletSync,
    puzzle_lookup_task: Option<JoinHandle<Result<(), WalletError>>>,
    nft_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    transaction_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    pending_coin_subscriptions: Vec<Bytes32>,
}

impl fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncManager").finish()
    }
}

#[derive(Default)]
enum InitialWalletSync {
    #[default]
    Idle,
    Syncing {
        ip: IpAddr,
        task: JoinHandle<Result<(), WalletError>>,
    },
    Subscribed(IpAddr),
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        if let InitialWalletSync::Syncing { task, .. } = &mut self.initial_wallet_sync {
            task.abort();
        }
        if let Some(task) = &mut self.puzzle_lookup_task {
            task.abort();
        }
        if let Some(task) = &mut self.nft_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.transaction_queue_task {
            task.abort();
        }
    }
}

impl SyncManager {
    pub fn new(
        options: SyncOptions,
        state: Arc<Mutex<PeerState>>,
        wallet: Option<Arc<Wallet>>,
        network_id: String,
        network: Network,
        connector: Connector,
    ) -> (Self, mpsc::Sender<SyncCommand>, mpsc::Receiver<SyncEvent>) {
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);

        let manager = Self {
            options,
            state,
            wallet,
            network_id,
            network,
            connector,
            event_sender,
            command_sender: command_sender.clone(),
            command_receiver,
            initial_wallet_sync: InitialWalletSync::Idle,
            puzzle_lookup_task: None,
            nft_queue_task: None,
            transaction_queue_task: None,
            pending_coin_subscriptions: Vec::new(),
        };

        (manager, command_sender, event_receiver)
    }

    pub async fn sync(mut self) {
        loop {
            self.process_commands().await;
            self.update().await;
            self.subscribe().await;
            sleep(self.options.sync_delay).await;
        }
    }

    async fn process_commands(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                SyncCommand::SwitchWallet { wallet } => {
                    self.clear_subscriptions().await;
                    self.abort_wallet_tasks();
                    self.wallet = wallet;
                }
                SyncCommand::SwitchNetwork {
                    network_id,
                    network,
                } => {
                    self.state.lock().await.reset();
                    self.abort_wallet_tasks();
                    self.network_id = network_id;
                    self.network = network;
                }
                SyncCommand::HandleMessage { ip, message } => {
                    if let Err(error) = self.handle_message(ip, message).await {
                        debug!("Failed to handle message from {ip}: {error}");
                        self.state.lock().await.ban(ip, Duration::from_secs(300));
                    }
                }
                SyncCommand::ConnectPeer { ip, trusted } => {
                    if trusted {
                        self.state.lock().await.trust(ip);
                    }

                    self.connect_batch(&[SocketAddr::new(ip, self.network.default_port)])
                        .await;
                }
                SyncCommand::SubscribeCoin { coin_id } => {
                    self.pending_coin_subscriptions.push(coin_id);
                }
                SyncCommand::ConnectionClosed(ip) => {
                    self.state.lock().await.remove_peer(ip);
                    debug!("Peer {ip} disconnected");
                }
                SyncCommand::SetDiscoverPeers(discover_peers) => {
                    self.options.discover_peers = discover_peers;
                }
                SyncCommand::SetTargetPeers(target_peers) => {
                    self.options.target_peers = target_peers;
                }
            }
        }
    }

    async fn subscribe(&mut self) {
        if self.pending_coin_subscriptions.is_empty() {
            return;
        }

        if let InitialWalletSync::Subscribed(ip) = self.initial_wallet_sync {
            if let Some(info) = self.state.lock().await.peer(ip) {
                debug!(
                    "Subscribing to {} new coin ids",
                    self.pending_coin_subscriptions.len()
                );

                // TODO: Handle cases
                timeout(
                    Duration::from_secs(3),
                    info.peer.request_coin_state(
                        mem::take(&mut self.pending_coin_subscriptions),
                        None,
                        self.network.genesis_challenge,
                        true,
                    ),
                )
                .await
                .map(Result::ok)
                .ok();
            }
        }
    }

    fn abort_wallet_tasks(&mut self) {
        if let InitialWalletSync::Syncing { task, .. } =
            std::mem::take(&mut self.initial_wallet_sync)
        {
            task.abort();
        }
        if let Some(task) = self.puzzle_lookup_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.nft_queue_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.transaction_queue_task.take() {
            task.abort();
        }
    }

    async fn handle_message(&self, ip: IpAddr, message: Message) -> Result<(), WalletError> {
        match message.msg_type {
            ProtocolMessageTypes::NewPeakWallet => {
                let message =
                    NewPeakWallet::from_bytes(&message.data).map_err(ClientError::from)?;
                self.state
                    .lock()
                    .await
                    .update_peak(ip, message.height, message.header_hash);
            }
            ProtocolMessageTypes::CoinStateUpdate => {
                let message =
                    CoinStateUpdate::from_bytes(&message.data).map_err(ClientError::from)?;
                if let Some(wallet) = self.wallet.as_ref() {
                    let unspent_count = message
                        .items
                        .iter()
                        .filter(|item| item.spent_height.is_none())
                        .count();

                    let spent_coin_ids = message
                        .items
                        .iter()
                        .filter_map(|item| {
                            if item.spent_height.is_some() {
                                Some(item.coin.coin_id())
                            } else {
                                None
                            }
                        })
                        .collect_vec();

                    let spent_count = spent_coin_ids.len();

                    if !spent_coin_ids.is_empty() {
                        if let InitialWalletSync::Subscribed(ip) = self.initial_wallet_sync {
                            if let Some(info) = self.state.lock().await.peer(ip) {
                                // TODO: Handle cases
                                timeout(
                                    Duration::from_secs(3),
                                    info.peer.remove_coin_subscriptions(Some(spent_coin_ids)),
                                )
                                .await
                                .map(Result::ok)
                                .ok();
                            }
                        }
                    }

                    incremental_sync(wallet, message.items, true, &self.event_sender).await?;

                    wallet
                        .db
                        .insert_peak(message.height, message.peak_hash)
                        .await?;

                    info!(
                        "Received {} unspent coins, {} spent coins, and synced to peak {} with header hash {}",
                        unspent_count, spent_count,
                        message.height, message.peak_hash
                    );
                } else {
                    debug!("Received coin state update but no database to update");
                }
            }
            _ => {
                debug!("Received unexpected message type: {:?}", message.msg_type);
            }
        }

        Ok(())
    }

    async fn update(&mut self) {
        let peer_count = self.state.lock().await.peer_count();

        if self.options.discover_peers && peer_count < self.options.target_peers {
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

    async fn update_tasks(&mut self) {
        let state = self.state.lock().await;

        match &mut self.initial_wallet_sync {
            sync @ InitialWalletSync::Idle => {
                if let Some(wallet) = self.wallet.clone() {
                    if let Some(peer) = state.acquire_peer() {
                        let ip = peer.socket_addr().ip();
                        let task = tokio::spawn(sync_wallet(
                            wallet.clone(),
                            peer,
                            self.state.clone(),
                            self.event_sender.clone(),
                        ));
                        *sync = InitialWalletSync::Syncing { ip, task };
                        self.event_sender.send(SyncEvent::Start(ip)).await.ok();
                    }
                }
            }
            InitialWalletSync::Syncing { ip, task }
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                task.abort();
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.event_sender.send(SyncEvent::Stop).await.ok();
            }
            InitialWalletSync::Subscribed(ip)
                if !state.is_connected(*ip) || self.wallet.is_none() =>
            {
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.event_sender.send(SyncEvent::Stop).await.ok();
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
                        self.event_sender.clone(),
                        self.command_sender.clone(),
                    )
                    .start(),
                );
                self.puzzle_lookup_task = Some(task);
            }

            if self.nft_queue_task.is_none() {
                let task = tokio::spawn(
                    NftQueue::new(wallet.db.clone(), self.event_sender.clone()).start(),
                );
                self.nft_queue_task = Some(task);
            }

            if self.transaction_queue_task.is_none() {
                let task = tokio::spawn(
                    TransactionQueue::new(
                        wallet.db.clone(),
                        wallet.genesis_challenge,
                        self.state.clone(),
                        self.event_sender.clone(),
                    )
                    .start(),
                );
                self.transaction_queue_task = Some(task);
            }
        } else {
            self.puzzle_lookup_task = None;
            self.nft_queue_task = None;
            self.transaction_queue_task = None;
        }
    }

    async fn poll_tasks(&mut self) {
        if let InitialWalletSync::Syncing { ip, task } = &mut self.initial_wallet_sync {
            if let Ok(Some(result)) = timeout(Duration::from_secs(1), poll_once(task)).await {
                match result {
                    Ok(Ok(())) => {
                        self.initial_wallet_sync = InitialWalletSync::Subscribed(*ip);
                        self.event_sender.send(SyncEvent::Subscribed).await.ok();
                    }
                    Ok(Err(error)) => {
                        warn!("Initial wallet sync failed: {error}");
                        self.state.lock().await.ban(*ip, Duration::from_secs(300));
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.event_sender.send(SyncEvent::Stop).await.ok();
                    }
                    Err(_timeout) => {
                        warn!("Initial wallet sync timed out");
                        self.state.lock().await.ban(*ip, Duration::from_secs(300));
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.event_sender.send(SyncEvent::Stop).await.ok();
                    }
                }
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

        if let Some(task) = &mut self.transaction_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("Transaction queue failed with panic: {error}");
                    self.transaction_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("Transaction queue failed with error: {error}");
                    self.transaction_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.transaction_queue_task = None;
                }
                None => {}
            }
        }
    }
}
