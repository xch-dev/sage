use std::{
    collections::HashSet,
    fmt,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use chia_traits::Streamable;
use chia_wallet_sdk::{
    chia::protocol::{
        CoinStateUpdate, MempoolItemsAdded, MempoolItemsRemoved, Message, NewPeakWallet,
        ProtocolMessageTypes,
    },
    client::{ClientError, Connector},
    prelude::*,
};
use futures_lite::future::poll_once;
use indexmap::IndexSet;
use itertools::Itertools;
use mempool_updates::{
    AnnouncedMempoolItem, MAX_PENDING_MEMPOOL_ITEMS, discover_mempool_items, remove_mempool_items,
};
use sage_config::Network;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
use tracing::{debug, info, warn};
use wallet_sync::{add_new_subscriptions, incremental_sync, sync_wallet};

use crate::{
    BlockTimeQueue, CatQueue, NftUriQueue, OfferQueue, PuzzleQueue, TransactionQueue, Wallet,
    WalletError,
};

mod dns;
mod mempool_updates;
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
    network: Network,
    connector: Connector,
    event_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
    command_receiver: mpsc::Receiver<SyncCommand>,
    initial_wallet_sync: InitialWalletSync,
    puzzle_lookup_task: Option<JoinHandle<Result<(), WalletError>>>,
    cat_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    nft_uri_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    transaction_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    offer_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    blocktime_queue_task: Option<JoinHandle<Result<(), WalletError>>>,
    mempool_discovery_task: Option<JoinHandle<Result<(), WalletError>>>,
    pending_coin_subscriptions: Vec<Bytes32>,
    pending_puzzle_subscriptions: Vec<Bytes32>,
    /// Transactions announced by peers, which are fetched once the initial sync has finished,
    /// so that the coins they spend and the derivations they pay to are already known.
    pending_mempool_items: IndexSet<AnnouncedMempoolItem>,
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
        if let Some(task) = &mut self.mempool_discovery_task {
            task.abort();
        }
    }
}

impl SyncManager {
    pub fn new(
        options: SyncOptions,
        state: Arc<Mutex<PeerState>>,
        wallet: Option<Arc<Wallet>>,
        network: Network,
        connector: Connector,
    ) -> (Self, mpsc::Sender<SyncCommand>, mpsc::Receiver<SyncEvent>) {
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);

        let manager = Self {
            options,
            state,
            wallet,
            network,
            connector,
            event_sender,
            command_sender: command_sender.clone(),
            command_receiver,
            initial_wallet_sync: InitialWalletSync::Idle,
            puzzle_lookup_task: None,
            cat_queue_task: None,
            nft_uri_queue_task: None,
            transaction_queue_task: None,
            offer_queue_task: None,
            blocktime_queue_task: None,
            mempool_discovery_task: None,
            pending_coin_subscriptions: Vec::new(),
            pending_puzzle_subscriptions: Vec::new(),
            pending_mempool_items: IndexSet::new(),
        };

        (manager, command_sender, event_receiver)
    }

    pub async fn sync(mut self) {
        loop {
            self.process_commands().await;
            self.update().await;
            self.subscribe().await;
            self.start_mempool_discovery();
            sleep(self.options.timeouts.sync_delay).await;
        }
    }

    async fn process_commands(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                SyncCommand::SwitchWallet { wallet, delta_sync } => {
                    let previous_fingerprint =
                        self.wallet.as_ref().map(|wallet| wallet.fingerprint);
                    let fingerprint = wallet.as_ref().map(|wallet| wallet.fingerprint);

                    self.clear_subscriptions().await;
                    self.abort_wallet_tasks();

                    // Queued subscriptions belong to the previous wallet. Syncing them under the
                    // new one would put its coin states in the wrong database.
                    self.pending_coin_subscriptions.clear();
                    self.pending_puzzle_subscriptions.clear();
                    self.pending_mempool_items.clear();

                    self.wallet = wallet;
                    self.options.delta_sync = delta_sync;

                    if previous_fingerprint != fingerprint {
                        let _ = self
                            .event_sender
                            .send(SyncEvent::WalletChanged { fingerprint })
                            .await;
                    }
                }
                SyncCommand::SwitchNetwork(network) => {
                    if self.network.network_id() != network.network_id()
                        || self.network.genesis_challenge != network.genesis_challenge
                        || self.network.default_port != network.default_port
                    {
                        self.state.lock().await.reset();
                        self.abort_wallet_tasks();
                        self.pending_mempool_items.clear();
                        self.network = *network;
                        let _ = self
                            .event_sender
                            .send(SyncEvent::NetworkChanged {
                                network_id: self.network.network_id(),
                            })
                            .await;
                    }
                }
                SyncCommand::HandleMessage { ip, message } => {
                    if let Err(error) = self.handle_message(ip, message).await {
                        debug!("Failed to handle message from {ip}: {error}");
                        self.state.lock().await.ban(
                            ip,
                            Duration::from_mins(5),
                            "failed to handle message",
                        );
                    }
                }
                SyncCommand::ConnectPeer { ip, user_managed } => {
                    self.connect_batch(
                        &[SocketAddr::new(ip, self.network.default_port)],
                        true,
                        user_managed,
                    )
                    .await;
                }
                SyncCommand::AddPeer { peer, receiver } => {
                    self.try_add_peer(peer, receiver, true, false).await;
                }
                SyncCommand::SubscribeCoins { coin_ids } => {
                    self.pending_coin_subscriptions.extend(coin_ids);
                }
                SyncCommand::SubscribePuzzles { puzzle_hashes } => {
                    self.pending_puzzle_subscriptions.extend(puzzle_hashes);
                }
                SyncCommand::ConnectionClosed(ip) => {
                    self.state.lock().await.remove_peer(ip);
                    debug!("Peer {ip} disconnected");
                }
                SyncCommand::SetTargetPeers(target_peers) => {
                    self.options.target_peers = target_peers;
                }
                SyncCommand::SetDiscoverPeers(discover_peers) => {
                    self.options.discover_peers = discover_peers;
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
            .lock()
            .await
            .peer(ip)
            .map(|info| info.peer.clone())
        else {
            return;
        };

        let Some(wallet) = self.wallet.as_ref() else {
            return;
        };

        if let Err(error) = add_new_subscriptions(
            wallet,
            &peer,
            self.pending_coin_subscriptions.clone(),
            self.pending_puzzle_subscriptions.clone(),
            self.event_sender.clone(),
            self.command_sender.clone(),
        )
        .await
        {
            warn!("Failed to add new subscriptions: {error}");
            self.state.lock().await.ban(
                ip,
                Duration::from_mins(5),
                "failed to add new subscriptions",
            );
        } else {
            self.pending_coin_subscriptions.clear();
            self.pending_puzzle_subscriptions.clear();
        }
    }

    fn queue_mempool_items(&mut self, ip: IpAddr, transaction_ids: Vec<Bytes32>) {
        for transaction_id in transaction_ids {
            if self.pending_mempool_items.len() >= MAX_PENDING_MEMPOOL_ITEMS {
                debug!("Too many pending mempool items, ignoring the rest from peer {ip}");
                break;
            }

            self.pending_mempool_items
                .insert(AnnouncedMempoolItem { ip, transaction_id });
        }
    }

    fn start_mempool_discovery(&mut self) {
        if self.pending_mempool_items.is_empty() || self.mempool_discovery_task.is_some() {
            return;
        }

        // Until the initial sync finishes, the wallet may not know the coins a transaction
        // spends, or have derived the addresses it pays to.
        if !matches!(self.initial_wallet_sync, InitialWalletSync::Subscribed(_)) {
            return;
        }

        let Some(wallet) = self.wallet.clone() else {
            return;
        };

        let items = std::mem::take(&mut self.pending_mempool_items)
            .into_iter()
            .collect();

        self.mempool_discovery_task = Some(tokio::spawn(discover_mempool_items(
            wallet,
            self.state.clone(),
            items,
            self.event_sender.clone(),
            self.command_sender.clone(),
        )));
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
        if let Some(task) = self.mempool_discovery_task.take() {
            task.abort();
        }
    }

    async fn handle_message(&mut self, ip: IpAddr, message: Message) -> Result<(), WalletError> {
        match message.msg_type {
            ProtocolMessageTypes::NewPeakWallet => {
                let message =
                    NewPeakWallet::from_bytes(&message.data).map_err(ClientError::from)?;

                let previous_peak = {
                    let mut state = self.state.lock().await;
                    let previous_peak = state.peak_of(ip).map(|(height, _)| height);
                    state.update_peak(ip, message.height, message.header_hash);
                    previous_peak
                };

                // When the chain is extended, the fork point is the previous peak. A fork point
                // below it means blocks were rolled back, so handle mappings verified against
                // them must be verified again. The peer can only have rolled back blocks up to
                // its previous peak, so a lagging peer doesn't affect anything verified above.
                if let Some(wallet) = self.wallet.as_ref()
                    && let Some(previous_peak) = previous_peak
                    && message.fork_point_with_previous_peak < previous_peak
                    && let Err(error) = wallet
                        .shared_db
                        .invalidate_xchandles_between(
                            message.fork_point_with_previous_peak,
                            previous_peak,
                        )
                        .await
                {
                    warn!("Failed to invalidate handle mappings after a reorg: {error}");
                }
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

                    if !spent_coin_ids.is_empty()
                        && let InitialWalletSync::Subscribed(ip) = self.initial_wallet_sync
                        && let Some(info) = self.state.lock().await.peer(ip)
                    {
                        // TODO: Handle cases
                        info.peer.unsubscribe_coins(spent_coin_ids).await.ok();
                    }

                    incremental_sync(
                        wallet,
                        message.items,
                        true,
                        &self.event_sender,
                        &self.command_sender,
                    )
                    .await?;

                    info!(
                        "Received {} unspent coins, {} spent coins, and synced to peak {} with header hash {}",
                        unspent_count, spent_count, message.height, message.peak_hash
                    );
                } else {
                    debug!("Received coin state update but no database to update");
                }
            }
            ProtocolMessageTypes::MempoolItemsAdded => {
                let message =
                    MempoolItemsAdded::from_bytes(&message.data).map_err(ClientError::from)?;
                self.queue_mempool_items(ip, message.transaction_ids);
            }
            ProtocolMessageTypes::MempoolItemsRemoved => {
                let message =
                    MempoolItemsRemoved::from_bytes(&message.data).map_err(ClientError::from)?;

                // An item that's still waiting to be fetched would only be removed again.
                let removed: HashSet<Bytes32> = message
                    .removed_items
                    .iter()
                    .map(|item| item.transaction_id)
                    .collect();
                self.pending_mempool_items
                    .retain(|item| !removed.contains(&item.transaction_id));

                if let Some(wallet) = self.wallet.as_ref() {
                    remove_mempool_items(wallet, message.removed_items, &self.event_sender).await?;
                }
            }
            _ => {
                debug!("Received unexpected message type: {:?}", message.msg_type);
            }
        }

        Ok(())
    }

    async fn update(&mut self) {
        self.drop_silent_peers().await;

        let peer_count = self.state.lock().await.peer_count();

        if peer_count < self.options.target_peers && self.options.discover_peers {
            if peer_count > 0 {
                if !self.peer_discovery().await && !self.dns_discovery().await {
                    self.introducer_discovery().await;
                }
            } else if !self.dns_discovery().await {
                self.introducer_discovery().await;
            }
        }

        self.update_tasks().await;
        self.poll_tasks().await;
    }

    /// Removes silent peers without banning them. After the computer wakes from sleep, live
    /// peers can look silent too, and a fresh connection to one is fine.
    async fn drop_silent_peers(&self) {
        if !self.network.drop_silent_peers {
            return;
        }

        let mut state = self.state.lock().await;

        for ip in state.silent_peers(Instant::now(), self.options.timeouts.peer_silence) {
            info!("Dropping peer {ip}, which stopped sending new peaks");
            state.remove_peer(ip);
        }
    }

    async fn update_tasks(&mut self) {
        let state = self.state.lock().await;

        match &mut self.initial_wallet_sync {
            sync @ InitialWalletSync::Idle => {
                if let Some(wallet) = self.wallet.clone()
                    && let Some(peer) = state.acquire_peer()
                {
                    let ip = peer.socket_addr().ip();
                    let task = tokio::spawn(sync_wallet(
                        wallet.clone(),
                        peer,
                        self.state.clone(),
                        self.event_sender.clone(),
                        self.command_sender.clone(),
                        self.options.delta_sync,
                    ));
                    *sync = InitialWalletSync::Syncing { ip, task };
                    self.event_sender.send(SyncEvent::Start(ip)).await.ok();
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
                        self.options.puzzle_batch_size_per_peer,
                        self.state.clone(),
                        self.event_sender.clone(),
                        self.command_sender.clone(),
                    )
                    .start(self.options.timeouts.puzzle_delay),
                );
                self.puzzle_lookup_task = Some(task);
            }

            if self.cat_queue_task.is_none() && !self.options.testing {
                let mainnet = self.network.genesis_challenge == MAINNET_CONSTANTS.genesis_challenge;
                let testnet =
                    self.network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

                if mainnet || testnet {
                    let task = tokio::spawn(
                        CatQueue::new(wallet.db.clone(), testnet, self.event_sender.clone())
                            .start(self.options.timeouts.cat_delay),
                    );
                    self.cat_queue_task = Some(task);
                }
            }

            if self.nft_uri_queue_task.is_none() && !self.options.testing {
                let task = tokio::spawn(
                    NftUriQueue::new(
                        wallet.db.clone(),
                        self.event_sender.clone(),
                        self.network.clone(),
                    )
                    .start(self.options.timeouts.nft_uri_delay),
                );
                self.nft_uri_queue_task = Some(task);
            }

            if self.transaction_queue_task.is_none() {
                let task = tokio::spawn(
                    TransactionQueue::new(
                        wallet.db.clone(),
                        self.state.clone(),
                        self.event_sender.clone(),
                    )
                    .start(self.options.timeouts.transaction_delay),
                );
                self.transaction_queue_task = Some(task);
            }

            if self.offer_queue_task.is_none() {
                let task = tokio::spawn(
                    OfferQueue::new(
                        wallet.db.clone(),
                        wallet.genesis_challenge,
                        self.state.clone(),
                        self.event_sender.clone(),
                    )
                    .start(self.options.timeouts.offer_delay),
                );
                self.offer_queue_task = Some(task);
            }

            if self.blocktime_queue_task.is_none() && !self.options.testing {
                let task = tokio::spawn(
                    BlockTimeQueue::new(
                        wallet.db.clone(),
                        self.state.clone(),
                        self.event_sender.clone(),
                    )
                    .start(self.options.timeouts.blocktime_delay),
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
            self.mempool_discovery_task = None;
        }
    }

    async fn poll_tasks(&mut self) {
        // Set when initial sync just finished, so the analyze below happens after
        // the borrow of `initial_wallet_sync` has been released.
        let mut initial_sync_completed = false;

        if let InitialWalletSync::Syncing { ip, task } = &mut self.initial_wallet_sync
            && let Ok(Some(result)) = timeout(Duration::from_secs(1), poll_once(task)).await
        {
            match result {
                Ok(Ok(())) => {
                    self.initial_wallet_sync = InitialWalletSync::Subscribed(*ip);
                    self.event_sender.send(SyncEvent::Subscribed).await.ok();
                    initial_sync_completed = true;
                }
                Ok(Err(error)) => {
                    warn!("Initial wallet sync failed: {error}");
                    self.state
                        .lock()
                        .await
                        .ban(*ip, Duration::from_mins(5), "wallet sync failed");
                    self.initial_wallet_sync = InitialWalletSync::Idle;
                    self.event_sender.send(SyncEvent::Stop).await.ok();
                }
                Err(_timeout) => {
                    warn!("Initial wallet sync timed out");
                    self.state.lock().await.ban(
                        *ip,
                        Duration::from_mins(5),
                        "wallet sync timed out",
                    );
                    self.initial_wallet_sync = InitialWalletSync::Idle;
                    self.event_sender.send(SyncEvent::Stop).await.ok();
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

        if let Some(task) = &mut self.mempool_discovery_task
            && let Some(result) = poll_once(task).await
        {
            match result {
                Err(error) => warn!("Mempool discovery failed with panic: {error}"),
                Ok(Err(error)) => warn!("Mempool discovery failed with error: {error}"),
                Ok(Ok(())) => {}
            }
            self.mempool_discovery_task = None;
        }

        // The startup analyze writes nothing for a wallet whose tables were still
        // empty, so a first sync would otherwise run with no statistics at all.
        if initial_sync_completed && let Some(wallet) = self.wallet.clone() {
            if let Err(error) = wallet.db.analyze().await {
                warn!("Failed to refresh query planner statistics: {error}");
            } else {
                info!("Refreshed query planner statistics after initial sync");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::{client::Connector, test::PeerSimulator};
    use sage_config::TESTNET11;

    use crate::default_test_options;

    use super::*;

    #[tokio::test]
    async fn switching_wallets_drops_pending_subscriptions() -> anyhow::Result<()> {
        let options = SyncOptions {
            target_peers: 0,
            discover_peers: false,
            dns_batch_size: 0,
            connection_batch_size: 0,
            max_peer_age_seconds: 0,
            delta_sync: true,
            puzzle_batch_size_per_peer: 0,
            timeouts: Timeouts::default(),
            testing: true,
        };

        let (mut manager, sender, _events) = SyncManager::new(
            options,
            Arc::new(Mutex::new(PeerState::default())),
            None,
            TESTNET11.clone(),
            Connector::Plain,
        );

        sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: vec![Bytes32::new([1; 32])],
            })
            .await?;
        sender
            .send(SyncCommand::SubscribePuzzles {
                puzzle_hashes: vec![Bytes32::new([2; 32])],
            })
            .await?;
        manager.process_commands().await;

        assert_eq!(manager.pending_coin_subscriptions.len(), 1);
        assert_eq!(manager.pending_puzzle_subscriptions.len(), 1);

        sender
            .send(SyncCommand::SwitchWallet {
                wallet: None,
                delta_sync: true,
            })
            .await?;
        manager.process_commands().await;

        assert!(manager.pending_coin_subscriptions.is_empty());
        assert!(manager.pending_puzzle_subscriptions.is_empty());

        Ok(())
    }

    /// Returns the simulator too, so the peer's connection stays open.
    async fn manager_with_sim_peer(
        network: Network,
    ) -> anyhow::Result<(SyncManager, PeerSimulator)> {
        let mut options = default_test_options();
        options.timeouts.peer_silence = Duration::ZERO;

        let (mut manager, _sender, _events) = SyncManager::new(
            options,
            Arc::new(Mutex::new(PeerState::default())),
            None,
            network,
            Connector::Plain,
        );

        let sim = PeerSimulator::new().await?;
        let (peer, receiver) = sim.connect_raw().await?;
        assert!(manager.try_add_peer(peer, receiver, true, false).await);

        Ok((manager, sim))
    }

    #[tokio::test]
    async fn silent_peer_is_dropped() -> anyhow::Result<()> {
        let (manager, _sim) = manager_with_sim_peer(TESTNET11.clone()).await?;

        manager.drop_silent_peers().await;

        assert_eq!(manager.state.lock().await.peer_count(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn silent_peer_is_kept_when_disabled_for_network() -> anyhow::Result<()> {
        let (manager, _sim) = manager_with_sim_peer(Network {
            drop_silent_peers: false,
            ..TESTNET11.clone()
        })
        .await?;

        manager.drop_silent_peers().await;

        assert_eq!(manager.state.lock().await.peer_count(), 1);

        Ok(())
    }
}
