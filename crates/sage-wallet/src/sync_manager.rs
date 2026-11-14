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
use indexmap::IndexMap;
use itertools::Itertools;
use mempool_updates::{
    AnnouncedMempoolItem, FinalRemovals, MAX_PENDING_MEMPOOL_ITEMS, MEMPOOL_RETRY_DELAYS,
    MempoolLookup, UnfinishedLookups, discover_mempool_items, remove_mempool_items,
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
mod mempool_source;
mod mempool_updates;
mod options;
mod peer_discovery;
mod peer_state;
mod sync_command;
mod sync_event;
mod wallet_sync;

pub use mempool_source::*;
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
    mempool_discovery_task: Option<JoinHandle<UnfinishedLookups>>,
    pending_coin_subscriptions: Vec<Bytes32>,
    pending_puzzle_subscriptions: Vec<Bytes32>,
    /// Transactions announced by peers, by id, and when to look each one up. They're looked up
    /// once the initial sync has finished, so that the coins they spend and the derivations they
    /// pay to are already known. New ones are due right away, and ones that weren't found are
    /// due again after a backoff.
    mempool_queue: IndexMap<Bytes32, (Instant, MempoolLookup)>,
    /// The transactions the running discovery batch is looking up. One that leaves the mempool
    /// in the meantime, for any reason, is taken out, so its lookup isn't tried again.
    mempool_in_flight: HashSet<Bytes32>,
    final_mempool_removals: FinalRemovals,
    /// Where announced transactions are fetched from, or `None` if the network has nowhere.
    mempool_source: Option<Arc<dyn MempoolSource>>,
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

/// Tests never look anything up with the real Coinset API, and set their own source instead.
fn default_mempool_source(
    options: &SyncOptions,
    network: &Network,
) -> Option<Arc<dyn MempoolSource>> {
    if options.testing {
        None
    } else {
        coinset_mempool(network)
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
            mempool_source: default_mempool_source(&options, &network),
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
            mempool_queue: IndexMap::new(),
            mempool_in_flight: HashSet::new(),
            final_mempool_removals: FinalRemovals::default(),
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
                        self.network = *network;
                        self.mempool_source = default_mempool_source(&self.options, &self.network);
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
                SyncCommand::SetMempoolDiscovery(mempool_discovery) => {
                    self.set_mempool_discovery(mempool_discovery);
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

    /// The source to fetch announced transactions from, if mempool discovery is turned on and
    /// the network has one.
    fn active_mempool_source(&self) -> Option<&Arc<dyn MempoolSource>> {
        if !self.options.mempool_discovery {
            return None;
        }
        self.mempool_source.as_ref()
    }

    fn set_mempool_discovery(&mut self, mempool_discovery: bool) {
        self.options.mempool_discovery = mempool_discovery;

        // Nothing more is looked up once it's turned off. Transactions it misses are announced
        // again the next time the wallet subscribes, if it's turned back on.
        if !mempool_discovery {
            self.abort_mempool_discovery();
        }
    }

    #[cfg(test)]
    pub(crate) fn set_mempool_source(&mut self, mempool_source: Option<Arc<dyn MempoolSource>>) {
        self.mempool_source = mempool_source;
    }

    fn queue_mempool_items(&mut self, ip: IpAddr, transaction_ids: Vec<Bytes32>) {
        info!(
            "Peer {ip} announced {} mempool items for the wallet",
            transaction_ids.len()
        );

        if self.active_mempool_source().is_none() {
            debug!("Mempool discovery is off, so the announced items aren't looked up");
            return;
        }

        let now = Instant::now();

        for transaction_id in transaction_ids {
            // It's already queued, waiting to be looked up again, or being looked up.
            if self.mempool_queue.contains_key(&transaction_id)
                || self.mempool_in_flight.contains(&transaction_id)
            {
                continue;
            }

            if self.mempool_queue.len() + self.mempool_in_flight.len() >= MAX_PENDING_MEMPOOL_ITEMS
            {
                debug!("Too many pending mempool items, ignoring the rest from peer {ip}");
                break;
            }

            let item = AnnouncedMempoolItem { ip, transaction_id };
            self.mempool_queue
                .insert(transaction_id, (now, MempoolLookup { item, misses: 0 }));
        }
    }

    /// Stops the running discovery batch and drops everything queued.
    fn abort_mempool_discovery(&mut self) {
        if let Some(task) = self.mempool_discovery_task.take() {
            task.abort();
        }
        self.mempool_queue.clear();
        self.mempool_in_flight.clear();
    }

    /// Drops lookups of transactions that left the mempool, including ones the running batch
    /// is doing, so they aren't tried again.
    fn forget_mempool_items(&mut self, transaction_ids: &HashSet<Bytes32>) {
        self.mempool_queue
            .retain(|transaction_id, _| !transaction_ids.contains(transaction_id));
        self.mempool_in_flight
            .retain(|transaction_id| !transaction_ids.contains(transaction_id));
    }

    /// Queues another lookup of each transaction from the batch that wasn't found, after a
    /// backoff, until it has been tried as many times as there are retry delays. Lookups that
    /// were deferred without being tried keep their count.
    fn retry_mempool_lookups(&mut self, unfinished: UnfinishedLookups, now: Instant) {
        let missed = unfinished.missed.into_iter().map(|lookup| (lookup, true));
        let deferred = unfinished
            .deferred
            .into_iter()
            .map(|lookup| (lookup, false));

        for (lookup, tried) in missed.chain(deferred) {
            let transaction_id = lookup.item.transaction_id;

            // It left the mempool while it was being looked up, so it won't be found.
            if !self.mempool_in_flight.remove(&transaction_id) {
                continue;
            }

            let misses = if tried {
                lookup.misses + 1
            } else {
                lookup.misses
            };

            let Some(&delay) = MEMPOOL_RETRY_DELAYS.get(misses.saturating_sub(1)) else {
                info!(
                    "Giving up on mempool item {transaction_id}, which wasn't found after \
                     {misses} lookups"
                );
                continue;
            };

            if self.mempool_queue.len() + self.mempool_in_flight.len() >= MAX_PENDING_MEMPOOL_ITEMS
            {
                debug!("Too many pending mempool items, not looking up {transaction_id} again");
                continue;
            }

            if tried {
                info!(
                    "Mempool item {transaction_id} wasn't found, looking it up again in {}s",
                    delay.as_secs()
                );
            }

            self.mempool_queue.insert(
                transaction_id,
                (
                    now + delay,
                    MempoolLookup {
                        item: lookup.item,
                        misses,
                    },
                ),
            );
        }
    }

    /// Handles the end of a discovery batch.
    fn finish_mempool_discovery(&mut self, unfinished: UnfinishedLookups, now: Instant) {
        self.retry_mempool_lookups(unfinished, now);
        self.mempool_in_flight.clear();
    }

    fn start_mempool_discovery(&mut self) {
        self.start_mempool_discovery_at(Instant::now());
    }

    fn start_mempool_discovery_at(&mut self, now: Instant) {
        if self.mempool_discovery_task.is_some()
            || !self.mempool_queue.values().any(|(due, _)| *due <= now)
        {
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

        let Some(mempool_source) = self.active_mempool_source().cloned() else {
            self.abort_mempool_discovery();
            return;
        };

        let mut lookups = Vec::new();
        self.mempool_queue.retain(|_, (due, lookup)| {
            if *due <= now {
                lookups.push(*lookup);
                false
            } else {
                true
            }
        });

        self.mempool_in_flight = lookups
            .iter()
            .map(|lookup| lookup.item.transaction_id)
            .collect();

        // Removals that arrived before now were applied to the queue, so only ones that
        // arrive during this batch matter to it.
        self.final_mempool_removals
            .lock()
            .expect("final removals lock")
            .clear();

        self.mempool_discovery_task = Some(tokio::spawn(discover_mempool_items(
            wallet,
            self.state.clone(),
            mempool_source,
            lookups,
            self.final_mempool_removals.clone(),
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
        self.abort_mempool_discovery();
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
                self.forget_mempool_items(&removed);

                if let Some(wallet) = self.wallet.as_ref() {
                    remove_mempool_items(
                        wallet,
                        message.removed_items,
                        &self.final_mempool_removals,
                        &self.event_sender,
                    )
                    .await?;
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
                    && let Some(peer) = state.acquire_sync_peer()
                {
                    let ip = peer.socket_addr().ip();

                    if self.options.mempool_discovery
                        && !state
                            .peer(ip)
                            .is_some_and(|info| info.sends_mempool_updates)
                    {
                        info!(
                            "Peer {ip} doesn't send mempool updates, so pending transactions will \
                             only show up once they're confirmed"
                        );
                    }

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
            self.mempool_discovery_task = None;
            let unfinished = result.unwrap_or_else(|error| {
                warn!("Mempool discovery failed with panic: {error}");
                UnfinishedLookups::default()
            });
            self.finish_mempool_discovery(unfinished, Instant::now());
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
    use chia_wallet_sdk::{
        chia::protocol::{MempoolRemoveReason, RemovedMempoolItem},
        client::Connector,
        test::PeerSimulator,
    };
    use sage_config::TESTNET11;

    use crate::{FakeMempool, TestWallet, default_test_options};

    use super::*;

    #[tokio::test]
    async fn switching_wallets_drops_pending_subscriptions() -> anyhow::Result<()> {
        let options = SyncOptions {
            target_peers: 0,
            discover_peers: false,
            mempool_discovery: false,
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

    fn bare_manager(wallet: Option<Arc<Wallet>>) -> (SyncManager, mpsc::Sender<SyncCommand>) {
        let (mut manager, sender, _events) = SyncManager::new(
            default_test_options(),
            Arc::new(Mutex::new(PeerState::default())),
            wallet,
            TESTNET11.clone(),
            Connector::Plain,
        );
        manager.set_mempool_source(Some(Arc::new(FakeMempool::default())));
        (manager, sender)
    }

    fn id(byte: u8) -> Bytes32 {
        Bytes32::new([byte; 32])
    }

    fn lookup(ip: IpAddr, byte: u8, misses: usize) -> MempoolLookup {
        MempoolLookup {
            item: AnnouncedMempoolItem {
                ip,
                transaction_id: id(byte),
            },
            misses,
        }
    }

    fn missed(lookups: Vec<MempoolLookup>) -> UnfinishedLookups {
        UnfinishedLookups {
            missed: lookups,
            deferred: Vec::new(),
        }
    }

    /// Queues retries of lookups, as if a batch had just missed them.
    fn miss(manager: &mut SyncManager, lookups: Vec<MempoolLookup>, now: Instant) {
        manager
            .mempool_in_flight
            .extend(lookups.iter().map(|lookup| lookup.item.transaction_id));
        manager.retry_mempool_lookups(missed(lookups), now);
    }

    fn queued(manager: &SyncManager) -> Vec<Bytes32> {
        manager.mempool_queue.keys().copied().collect()
    }

    fn removal(byte: u8, reason: MempoolRemoveReason) -> anyhow::Result<Message> {
        Ok(Message {
            msg_type: ProtocolMessageTypes::MempoolItemsRemoved,
            id: None,
            data: MempoolItemsRemoved::new(vec![RemovedMempoolItem::new(id(byte), reason)])
                .to_bytes()?
                .into(),
        })
    }

    /// Makes the manager look like a discovery batch for `in_flight` is running.
    fn start_fake_batch(manager: &mut SyncManager, in_flight: &[u8]) {
        manager.mempool_discovery_task = Some(tokio::spawn(std::future::pending()));
        manager.mempool_in_flight = in_flight.iter().map(|&byte| id(byte)).collect();
    }

    #[tokio::test]
    async fn announced_mempool_items_are_queued_once_and_capped() -> anyhow::Result<()> {
        let (mut manager, _sender) = bare_manager(None);
        let ip = IpAddr::from([10, 0, 0, 1]);

        let message = Message {
            msg_type: ProtocolMessageTypes::MempoolItemsAdded,
            id: None,
            data: MempoolItemsAdded::new(vec![id(1), id(1)])
                .to_bytes()?
                .into(),
        };
        manager.handle_message(ip, message).await?;
        assert_eq!(queued(&manager), [id(1)]);

        // Another peer announcing it doesn't queue it twice.
        manager.queue_mempool_items(IpAddr::from([10, 0, 0, 2]), vec![id(1)]);
        assert_eq!(queued(&manager), [id(1)]);
        assert_eq!(manager.mempool_queue[&id(1)].1.item.ip, ip);

        let many = (0..u16::try_from(MAX_PENDING_MEMPOOL_ITEMS)? + 10)
            .map(|i| {
                let mut bytes = [0; 32];
                bytes[..2].copy_from_slice(&i.to_be_bytes());
                bytes[31] = 2;
                Bytes32::new(bytes)
            })
            .collect();
        manager.queue_mempool_items(ip, many);
        assert_eq!(manager.mempool_queue.len(), MAX_PENDING_MEMPOOL_ITEMS);

        Ok(())
    }

    #[tokio::test]
    async fn the_cap_counts_lookups_in_flight_and_retries() -> anyhow::Result<()> {
        let (mut manager, _sender) = bare_manager(None);
        let ip = IpAddr::from([10, 0, 0, 1]);

        start_fake_batch(&mut manager, &[1, 2]);
        miss(&mut manager, vec![lookup(ip, 3, 0)], Instant::now());

        let many: Vec<Bytes32> = (0..u16::try_from(MAX_PENDING_MEMPOOL_ITEMS)?)
            .map(|i| {
                let mut bytes = [0; 32];
                bytes[..2].copy_from_slice(&i.to_be_bytes());
                bytes[31] = 4;
                Bytes32::new(bytes)
            })
            .collect();
        manager.queue_mempool_items(ip, many);
        assert_eq!(manager.mempool_queue.len(), MAX_PENDING_MEMPOOL_ITEMS - 2);

        // The batch's misses take the room it held, and once the queue is full there's no room
        // for more retries.
        manager.finish_mempool_discovery(
            missed(vec![lookup(ip, 1, 0), lookup(ip, 2, 0)]),
            Instant::now(),
        );
        assert_eq!(manager.mempool_queue.len(), MAX_PENDING_MEMPOOL_ITEMS);

        miss(&mut manager, vec![lookup(ip, 5, 0)], Instant::now());
        assert!(!manager.mempool_queue.contains_key(&id(5)));

        Ok(())
    }

    #[tokio::test]
    async fn mempool_items_being_looked_up_are_not_queued_again() -> anyhow::Result<()> {
        let (mut manager, _sender) = bare_manager(None);
        let ip = IpAddr::from([10, 0, 0, 1]);

        start_fake_batch(&mut manager, &[1]);
        manager.queue_mempool_items(ip, vec![id(1), id(2)]);
        assert_eq!(queued(&manager), [id(2)]);

        Ok(())
    }

    #[tokio::test]
    async fn removed_mempool_items_are_dropped_from_the_queue() -> anyhow::Result<()> {
        let (mut manager, _sender) = bare_manager(None);
        let ip = IpAddr::from([10, 0, 0, 1]);

        manager.queue_mempool_items(ip, vec![id(1), id(2)]);
        miss(&mut manager, vec![lookup(ip, 3, 0)], Instant::now());

        for byte in [1, 3] {
            manager
                .handle_message(ip, removal(byte, MempoolRemoveReason::BlockInclusion)?)
                .await?;
        }

        assert_eq!(queued(&manager), [id(2)]);

        Ok(())
    }

    #[tokio::test]
    async fn mempool_items_removed_during_a_batch_are_not_looked_up_again() -> anyhow::Result<()> {
        let (mut manager, _sender) = bare_manager(None);
        let ip = IpAddr::from([10, 0, 0, 1]);

        // Being included in a block is the usual way a transaction leaves the mempool, and
        // the lookup that was running when it did can't find it any more.
        start_fake_batch(&mut manager, &[1, 2]);
        manager
            .handle_message(ip, removal(1, MempoolRemoveReason::BlockInclusion)?)
            .await?;

        manager.finish_mempool_discovery(
            missed(vec![lookup(ip, 1, 0), lookup(ip, 2, 0)]),
            Instant::now(),
        );
        assert_eq!(queued(&manager), [id(2)]);
        assert!(manager.mempool_in_flight.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn mempool_items_are_fetched_only_after_the_initial_sync() -> anyhow::Result<()> {
        let test = TestWallet::new(0).await?;
        let (mut manager, _sender) = bare_manager(Some(test.wallet.clone()));
        let ip = IpAddr::from([10, 0, 0, 1]);

        manager.queue_mempool_items(ip, vec![id(1)]);

        manager.start_mempool_discovery();
        assert!(manager.mempool_discovery_task.is_none());
        assert_eq!(queued(&manager), [id(1)]);

        manager.initial_wallet_sync = InitialWalletSync::Syncing {
            ip,
            task: tokio::spawn(std::future::pending()),
        };
        manager.start_mempool_discovery();
        assert!(manager.mempool_discovery_task.is_none());
        assert_eq!(queued(&manager), [id(1)]);

        manager.initial_wallet_sync = InitialWalletSync::Subscribed(ip);
        manager.start_mempool_discovery();
        assert!(manager.mempool_discovery_task.is_some());
        assert!(manager.mempool_queue.is_empty());
        assert_eq!(manager.mempool_in_flight, HashSet::from([id(1)]));

        Ok(())
    }

    #[tokio::test]
    async fn switching_wallets_or_networks_drops_queued_mempool_items() -> anyhow::Result<()> {
        let ip = IpAddr::from([10, 0, 0, 1]);

        let custom = Network {
            name: "custom".to_string(),
            network_id: Some("custom".to_string()),
            genesis_challenge: Bytes32::new([1; 32]),
            ..TESTNET11.clone()
        };

        for command in [
            SyncCommand::SwitchWallet {
                wallet: None,
                delta_sync: true,
            },
            SyncCommand::SwitchNetwork(Box::new(custom)),
        ] {
            let (mut manager, sender) = bare_manager(None);
            manager.queue_mempool_items(ip, vec![id(1)]);
            miss(&mut manager, vec![lookup(ip, 2, 0)], Instant::now());
            start_fake_batch(&mut manager, &[3]);

            sender.send(command).await?;
            manager.process_commands().await;

            assert!(manager.mempool_queue.is_empty());
            assert!(manager.mempool_in_flight.is_empty());
            assert!(manager.mempool_discovery_task.is_none());
        }

        Ok(())
    }

    #[tokio::test]
    async fn missed_mempool_lookups_are_retried_with_backoff_then_dropped() -> anyhow::Result<()> {
        let test = TestWallet::new(0).await?;
        let (mut manager, _sender) = bare_manager(Some(test.wallet.clone()));
        let ip = IpAddr::from([10, 0, 0, 1]);
        manager.initial_wallet_sync = InitialWalletSync::Subscribed(ip);
        let now = Instant::now();

        miss(&mut manager, vec![lookup(ip, 1, 0)], now);
        assert_eq!(
            manager.mempool_queue[&id(1)],
            (now + MEMPOOL_RETRY_DELAYS[0], lookup(ip, 1, 1))
        );

        // Announcing it again doesn't look it up any sooner.
        manager.queue_mempool_items(ip, vec![id(1)]);
        assert_eq!(
            manager.mempool_queue[&id(1)].0,
            now + MEMPOOL_RETRY_DELAYS[0]
        );

        manager.start_mempool_discovery_at(now);
        assert!(manager.mempool_discovery_task.is_none());

        manager.start_mempool_discovery_at(now + MEMPOOL_RETRY_DELAYS[0]);
        assert!(manager.mempool_discovery_task.is_some());
        assert!(manager.mempool_queue.is_empty());
        if let Some(task) = manager.mempool_discovery_task.take() {
            task.abort();
        }

        // Each miss waits longer, until it has been tried after every delay.
        manager.finish_mempool_discovery(missed(vec![lookup(ip, 1, 1)]), now);
        assert_eq!(
            manager.mempool_queue[&id(1)],
            (now + MEMPOOL_RETRY_DELAYS[1], lookup(ip, 1, 2))
        );
        manager.mempool_queue.clear();

        let last = MEMPOOL_RETRY_DELAYS.len() - 1;
        miss(&mut manager, vec![lookup(ip, 1, last)], now);
        assert_eq!(
            manager.mempool_queue[&id(1)],
            (now + MEMPOOL_RETRY_DELAYS[last], lookup(ip, 1, last + 1))
        );
        manager.mempool_queue.clear();

        miss(
            &mut manager,
            vec![lookup(ip, 1, MEMPOOL_RETRY_DELAYS.len())],
            now,
        );
        assert!(manager.mempool_queue.is_empty());

        // Lookups deferred without being tried keep their count.
        manager.mempool_in_flight.insert(id(2));
        manager.retry_mempool_lookups(
            UnfinishedLookups {
                missed: Vec::new(),
                deferred: vec![lookup(ip, 2, 2)],
            },
            now,
        );
        assert_eq!(
            manager.mempool_queue[&id(2)],
            (now + MEMPOOL_RETRY_DELAYS[1], lookup(ip, 2, 2))
        );

        Ok(())
    }

    #[tokio::test]
    async fn announced_mempool_items_are_ignored_without_mempool_discovery() -> anyhow::Result<()> {
        let test = TestWallet::new(0).await?;
        let (mut manager, sender) = bare_manager(Some(test.wallet.clone()));
        let ip = IpAddr::from([10, 0, 0, 1]);
        manager.initial_wallet_sync = InitialWalletSync::Subscribed(ip);

        // Turning it off drops what's queued and stops the batch that's looking things up.
        manager.queue_mempool_items(ip, vec![id(1)]);
        miss(&mut manager, vec![lookup(ip, 2, 0)], Instant::now());
        start_fake_batch(&mut manager, &[3]);
        sender.send(SyncCommand::SetMempoolDiscovery(false)).await?;
        manager.process_commands().await;
        assert!(manager.mempool_queue.is_empty());
        assert!(manager.mempool_in_flight.is_empty());
        assert!(manager.mempool_discovery_task.is_none());

        manager.queue_mempool_items(ip, vec![id(4)]);
        assert!(manager.mempool_queue.is_empty());

        // Nor is anything looked up on a network that has nowhere to look it up.
        sender.send(SyncCommand::SetMempoolDiscovery(true)).await?;
        manager.process_commands().await;
        manager.set_mempool_source(None);
        manager.queue_mempool_items(ip, vec![id(5)]);
        assert!(manager.mempool_queue.is_empty());

        manager.set_mempool_source(Some(Arc::new(FakeMempool::default())));
        manager.queue_mempool_items(ip, vec![id(6)]);
        assert_eq!(queued(&manager), [id(6)]);

        Ok(())
    }

    #[test]
    fn tests_never_use_the_real_coinset_api() {
        assert!(default_mempool_source(&default_test_options(), &TESTNET11).is_none());

        let mut options = default_test_options();
        options.testing = false;
        assert!(default_mempool_source(&options, &TESTNET11).is_some());
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
