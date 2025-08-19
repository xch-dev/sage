use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use chia::{
    protocol::{Bytes32, CoinStateUpdate, Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{
    client::{ClientError, Connector},
    types::{MAINNET_CONSTANTS, TESTNET11_CONSTANTS},
};
use futures_lite::future::poll_once;
use itertools::Itertools;
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{debug, info, warn};
use wallet_sync::{add_new_subscriptions, incremental_sync, sync_wallet};

use crate::{
    BlockTimeQueue, CatQueue, NftUriQueue, OfferQueue, PuzzleQueue, TransactionQueue, WalletError,
};

mod dns;
mod options;
mod peer_discovery;
mod peer_state;
mod sync_command;
mod sync_event;
mod sync_state;
mod wallet_sync;

pub use options::*;
pub use peer_state::*;
pub use sync_command::*;
pub use sync_event::*;
pub use sync_state::*;

pub struct SyncManager {
    state: SyncState,
    connector: Connector,
    command_receiver: mpsc::Receiver<SyncCommand>,
    initial_wallet_sync: InitialWalletSync,
    puzzle_lookup_task: Option<JoinHandle<Result<(), WalletError>>>,
    cat_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    nft_uri_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    transaction_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    offer_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    blocktime_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    pending_coin_subscriptions: Vec<Bytes32>,
    pending_puzzle_subscriptions: Vec<Bytes32>,
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
        if let Some(task) = &mut self.cat_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.nft_uri_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.transaction_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.offer_queue_task {
            task.abort();
        }
        if let Some(task) = &mut self.blocktime_queue_task {
            task.abort();
        }
    }
}

impl SyncManager {
    pub fn new(
        state: SyncState,
        command_receiver: mpsc::Receiver<SyncCommand>,
        connector: Connector,
    ) -> Self {
        Self {
            state,
            connector,
            command_receiver,
            initial_wallet_sync: InitialWalletSync::Idle,
            puzzle_lookup_task: None,
            cat_queue_task: None,
            nft_uri_queue_task: None,
            transaction_queue_task: None,
            offer_queue_task: None,
            blocktime_queue_task: None,
            pending_coin_subscriptions: Vec::new(),
            pending_puzzle_subscriptions: Vec::new(),
        }
    }

    pub async fn sync(mut self) {
        loop {
            self.process_commands().await;
            self.update().await;
            self.subscribe().await;
            sleep(self.state.options.timeouts.sync_delay).await;
        }
    }

    async fn process_commands(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                SyncCommand::Reset { remove_peers } => {
                    self.clear_subscriptions().await;
                    self.abort_wallet_tasks();

                    if remove_peers {
                        self.state.peers.lock().await.reset();
                    }
                }
                SyncCommand::HandleMessage { ip, message } => {
                    if let Err(error) = self.handle_message(ip, message).await {
                        debug!("Failed to handle message from {ip}: {error}");
                        self.state.peers.lock().await.ban(
                            ip,
                            Duration::from_secs(300),
                            "failed to handle message",
                        );
                    }
                }
                SyncCommand::ConnectPeer { ip, user_managed } => {
                    let network = self.state.network.lock().await.clone();
                    let network_config = self.state.network_config.lock().await.clone();
                    self.connect_batch(
                        &[SocketAddr::new(ip, network.default_port)],
                        true,
                        user_managed,
                        &network,
                        &network_config,
                    )
                    .await;
                }
                SyncCommand::SubscribeCoins { coin_ids } => {
                    self.pending_coin_subscriptions.extend(coin_ids);
                }
                SyncCommand::SubscribePuzzles { puzzle_hashes } => {
                    self.pending_puzzle_subscriptions.extend(puzzle_hashes);
                }
                SyncCommand::ConnectionClosed(ip) => {
                    self.state.peers.lock().await.remove_peer(ip);
                    debug!("Peer {ip} disconnected");
                }
            }
        }
    }

    async fn subscribe(&mut self) {
        if self.pending_coin_subscriptions.is_empty()
            && self.pending_puzzle_subscriptions.is_empty()
        {
            return;
        }

        let InitialWalletSync::Subscribed(ip) = self.initial_wallet_sync else {
            return;
        };

        let Some(peer) = self
            .state
            .peers
            .lock()
            .await
            .peer(ip)
            .map(|info| info.peer.clone())
        else {
            return;
        };

        let Some(wallet) = self.state.wallet.lock().await.clone() else {
            return;
        };

        if let Err(error) = add_new_subscriptions(
            &wallet,
            &peer,
            &self.state,
            self.pending_coin_subscriptions.clone(),
            self.pending_puzzle_subscriptions.clone(),
        )
        .await
        {
            warn!("Failed to add new subscriptions: {error}");
            self.state.peers.lock().await.ban(
                ip,
                Duration::from_secs(300),
                "failed to add new subscriptions",
            );
        } else {
            self.pending_coin_subscriptions.clear();
            self.pending_puzzle_subscriptions.clear();
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
        if let Some(task) = &mut self.cat_queue_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.nft_uri_queue_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.transaction_queue_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.offer_queue_task.take() {
            task.abort();
        }
        if let Some(task) = &mut self.blocktime_queue_task.take() {
            task.abort();
        }
    }

    async fn handle_message(&self, ip: IpAddr, message: Message) -> Result<(), WalletError> {
        match message.msg_type {
            ProtocolMessageTypes::NewPeakWallet => {
                let message =
                    NewPeakWallet::from_bytes(&message.data).map_err(ClientError::from)?;
                self.state
                    .peers
                    .lock()
                    .await
                    .update_peak(ip, message.height, message.header_hash);
            }
            ProtocolMessageTypes::CoinStateUpdate => {
                let message =
                    CoinStateUpdate::from_bytes(&message.data).map_err(ClientError::from)?;

                if let Some(wallet) = self.state.wallet.lock().await.clone() {
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
                            if let Some(info) = self.state.peers.lock().await.peer(ip) {
                                // TODO: Handle cases
                                info.peer.unsubscribe_coins(spent_coin_ids).await.ok();
                            }
                        }
                    }

                    incremental_sync(&wallet, &self.state, message.items, true).await?;

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
        let peer_count = self.state.peers.lock().await.peer_count();

        let network = self.state.network.lock().await.clone();
        let network_config = self.state.network_config.lock().await.clone();

        if peer_count < network_config.target_peers && network_config.discover_peers {
            if peer_count > 0 {
                if !self.peer_discovery(&network, &network_config).await
                    && !self.dns_discovery(&network, &network_config).await
                {
                    self.introducer_discovery(&network, &network_config).await;
                }
            } else if !self.dns_discovery(&network, &network_config).await {
                self.introducer_discovery(&network, &network_config).await;
            }
        }

        self.update_tasks().await;
        self.poll_tasks().await;
    }

    async fn update_tasks(&mut self) {
        let state = self.state.peers.lock().await;

        let wallet = self.state.wallet.lock().await.clone();
        let wallet_config = self.state.wallet_config.lock().await.clone();
        let wallet_defaults = *self.state.wallet_defaults.lock().await;
        let network = self.state.network.lock().await.clone();

        match &mut self.initial_wallet_sync {
            sync @ InitialWalletSync::Idle => {
                if let Some(wallet) = wallet.clone() {
                    if let Some(peer) = state.acquire_peer() {
                        let ip = peer.socket_addr().ip();
                        let task = tokio::spawn(sync_wallet(
                            wallet,
                            peer,
                            self.state.clone(),
                            wallet_config.delta_sync(&wallet_defaults),
                        ));
                        *sync = InitialWalletSync::Syncing { ip, task };
                        self.state.events.send(SyncEvent::Start(ip)).await.ok();
                    }
                }
            }
            InitialWalletSync::Syncing { ip, task }
                if !state.is_connected(*ip) || wallet.is_none() =>
            {
                task.abort();
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.state.events.send(SyncEvent::Stop).await.ok();
            }
            InitialWalletSync::Subscribed(ip) if !state.is_connected(*ip) || wallet.is_none() => {
                self.initial_wallet_sync = InitialWalletSync::Idle;
                self.state.events.send(SyncEvent::Stop).await.ok();
            }
            _ => {}
        }

        if let Some(wallet) = wallet {
            if self.puzzle_lookup_task.is_none() {
                let task = tokio::spawn(
                    PuzzleQueue::new(
                        wallet.db.clone(),
                        self.state.clone(),
                        wallet.genesis_challenge,
                        self.state.options.puzzle_batch_size_per_peer,
                    )
                    .start(self.state.options.timeouts.puzzle_delay),
                );
                self.puzzle_lookup_task = Some(task);
            }

            if self.cat_queue_task.is_none() && !self.state.options.testing {
                let mainnet = network.genesis_challenge == MAINNET_CONSTANTS.genesis_challenge;
                let testnet = network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

                if mainnet || testnet {
                    let task = tokio::spawn(
                        CatQueue::new(wallet.db.clone(), self.state.clone(), testnet)
                            .start(self.state.options.timeouts.cat_delay),
                    );
                    self.cat_queue_task = Some(task);
                }
            }

            if self.nft_uri_queue_task.is_none() && !self.state.options.testing {
                let task = tokio::spawn(
                    NftUriQueue::new(wallet.db.clone(), self.state.clone())
                        .start(self.state.options.timeouts.nft_uri_delay),
                );
                self.nft_uri_queue_task = Some(task);
            }

            if self.transaction_queue_task.is_none() {
                let task = tokio::spawn(
                    TransactionQueue::new(wallet.db.clone(), self.state.clone())
                        .start(self.state.options.timeouts.transaction_delay),
                );
                self.transaction_queue_task = Some(task);
            }

            if self.offer_queue_task.is_none() {
                let task = tokio::spawn(
                    OfferQueue::new(
                        wallet.db.clone(),
                        self.state.clone(),
                        wallet.genesis_challenge,
                    )
                    .start(self.state.options.timeouts.offer_delay),
                );
                self.offer_queue_task = Some(task);
            }

            if self.blocktime_queue_task.is_none() && !self.state.options.testing {
                let task = tokio::spawn(
                    BlockTimeQueue::new(wallet.db.clone(), self.state.clone())
                        .start(self.state.options.timeouts.blocktime_delay),
                );
                self.blocktime_queue_task = Some(task);
            }
        } else {
            self.puzzle_lookup_task = None;
            self.cat_queue_task = None;
            self.nft_uri_queue_task = None;
            self.transaction_queue_task = None;
            self.offer_queue_task = None;
            self.blocktime_queue_task = None;
        }
    }

    async fn poll_tasks(&mut self) {
        if let InitialWalletSync::Syncing { ip, task } = &mut self.initial_wallet_sync {
            if let Ok(Some(result)) = timeout(Duration::from_secs(1), poll_once(task)).await {
                match result {
                    Ok(Ok(())) => {
                        self.initial_wallet_sync = InitialWalletSync::Subscribed(*ip);
                        self.state.events.send(SyncEvent::Subscribed).await.ok();
                    }
                    Ok(Err(error)) => {
                        warn!("Initial wallet sync failed: {error}");
                        self.state.peers.lock().await.ban(
                            *ip,
                            Duration::from_secs(300),
                            "wallet sync failed",
                        );
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.state.events.send(SyncEvent::Stop).await.ok();
                    }
                    Err(_timeout) => {
                        warn!("Initial wallet sync timed out");
                        self.state.peers.lock().await.ban(
                            *ip,
                            Duration::from_secs(300),
                            "wallet sync timed out",
                        );
                        self.initial_wallet_sync = InitialWalletSync::Idle;
                        self.state.events.send(SyncEvent::Stop).await.ok();
                    }
                }
            }
        }

        if let Some(task) = &mut self.puzzle_lookup_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("Puzzle lookup queue failed with panic: {error}");
                    self.puzzle_lookup_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("Puzzle lookup queue failed with error: {error}");
                    self.puzzle_lookup_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.puzzle_lookup_task = None;
                }
                None => {}
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

        if let Some(task) = &mut self.nft_uri_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("NFT URI queue failed with panic: {error}");
                    self.nft_uri_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("NFT URI queue failed with error: {error}");
                    self.nft_uri_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.nft_uri_queue_task = None;
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

        if let Some(task) = &mut self.offer_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("Offer queue failed with panic: {error}");
                    self.offer_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("Offer queue failed with error: {error}");
                    self.offer_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.offer_queue_task = None;
                }
                None => {}
            }
        }

        if let Some(task) = &mut self.blocktime_queue_task {
            match poll_once(task).await {
                Some(Err(error)) => {
                    warn!("Blocktime queue failed with panic: {error}");
                    self.blocktime_queue_task = None;
                }
                Some(Ok(Err(error))) => {
                    warn!("Blocktime queue failed with error: {error}");
                    self.blocktime_queue_task = None;
                }
                Some(Ok(Ok(()))) => {
                    self.blocktime_queue_task = None;
                }
                None => {}
            }
        }
    }
}
