use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{bls::master_to_wallet_unhardened_intermediate, protocol::Bytes32};
use chia_wallet_sdk::{
    client::{create_rustls_connector, load_ssl_cert, Connector},
    signer::AggSigConstants,
    utils::Address,
};
use indexmap::IndexMap;
use sage_api::{Unit, XCH};
use sage_config::{
    migrate_config, migrate_networks, Config, Network, NetworkList, OldConfig, OldNetwork,
    WalletConfig,
};
use sage_database::Database;
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts, Wallet};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    ConnectOptions, SqlitePool,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, Level};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{
    filter::filter_fn, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry,
};

use crate::{peers::Peers, webhook_manager::WebhookManager, Error, Result};

#[derive(Debug)]
pub struct Sage {
    pub path: PathBuf,
    pub config: Config,
    pub wallet_config: WalletConfig,
    pub network_list: NetworkList,
    pub keychain: Keychain,
    pub wallet: Option<Arc<Wallet>>,
    pub peer_state: Arc<Mutex<PeerState>>,
    pub command_sender: mpsc::Sender<SyncCommand>,
    pub unit: Unit,
    pub webhook_manager: WebhookManager,
}

impl Sage {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            config: Config::default(),
            wallet_config: WalletConfig::default(),
            network_list: NetworkList::default(),
            keychain: Keychain::default(),
            wallet: None,
            peer_state: Arc::new(Mutex::new(PeerState::default())),
            command_sender: mpsc::channel(1).0,
            unit: XCH.clone(),
            webhook_manager: WebhookManager::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        fs::create_dir_all(&self.path)?;

        self.setup_keys()?;
        self.setup_config()?;
        self.setup_logging()?;
        self.setup_webhooks().await?;

        // Initialize webhook manager with current fingerprint and network
        self.webhook_manager
            .set_fingerprint(self.config.global.fingerprint)
            .await;
        self.webhook_manager.set_network(self.network_id()).await;

        let receiver = self.setup_sync_manager()?;
        self.setup_peers().await?;

        info!("Sage wallet initialized");

        Ok(receiver)
    }

    fn setup_logging(&mut self) -> Result<()> {
        let log_dir = self.path.join("log");

        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }

        let log_level: Level = self.config.global.log_level.parse()?;

        // Create rotated log file
        let log_file = Builder::new()
            .filename_prefix("app.log")
            .rotation(Rotation::DAILY)
            .max_log_files(3)
            .build(log_dir)?;

        // Common filter string
        let filter_string = format!("{log_level},rustls=off,tungstenite=off,h2=off,hyper=off");

        // File layer - always without ANSI
        let file_layer = fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(log_file)
            .compact()
            .with_filter(EnvFilter::new(&filter_string));

        // Terminal layer - with ANSI and additional formatting
        let terminal_layer = fmt::layer()
            .with_target(false)
            .with_ansi(true) // Explicitly enable ANSI for terminal
            .compact()
            .with_filter(EnvFilter::new(&filter_string));

        // Build subscriber differently based on platform
        let subscriber = Registry::default()
            .with(file_layer)
            .with(terminal_layer.with_filter(filter_fn(|_| !cfg!(mobile))));

        // Initialize the subscriber
        tracing::subscriber::set_global_default(subscriber)?;

        Ok(())
    }

    fn setup_keys(&mut self) -> Result<()> {
        let key_path = self.path.join("keys.bin");

        if key_path.try_exists()? {
            let data = fs::read(&key_path)?;
            self.keychain = Keychain::from_bytes(&data)?;
        } else {
            fs::write(&key_path, self.keychain.to_bytes()?)?;
        }

        Ok(())
    }

    fn setup_config(&mut self) -> Result<()> {
        let config_path = self.path.join("config.toml");
        let wallet_config_path = self.path.join("wallets.toml");
        let network_list_path = self.path.join("networks.toml");

        if config_path.try_exists()? {
            let config_text = fs::read_to_string(&config_path)?;

            if let Some(old_config) = toml::from_str::<OldConfig>(&config_text)
                .ok()
                .filter(OldConfig::is_old)
            {
                let (config, wallet_config) = migrate_config(old_config)?;
                self.config = config;
                self.wallet_config = wallet_config;
                fs::write(&config_path, toml::to_string_pretty(&self.config)?)?;
                fs::write(
                    &wallet_config_path,
                    toml::to_string_pretty(&self.wallet_config)?,
                )?;
            } else {
                self.config = toml::from_str(&config_text)?;
                let wallet_config_text = fs::read_to_string(&wallet_config_path)?;
                self.wallet_config = toml::from_str(&wallet_config_text)?;
            }
        } else {
            fs::write(&config_path, toml::to_string_pretty(&self.config)?)?;
            fs::write(
                &wallet_config_path,
                toml::to_string_pretty(&self.wallet_config)?,
            )?;
        }

        if network_list_path.try_exists()? {
            let text = fs::read_to_string(&network_list_path)?;

            if let Ok(old_network_list) = toml::from_str::<IndexMap<String, OldNetwork>>(&text) {
                self.network_list = migrate_networks(old_network_list);
                fs::write(
                    &network_list_path,
                    toml::to_string_pretty(&self.network_list)?,
                )?;
            } else {
                self.network_list = toml::from_str(&text)?;
            }
        } else {
            fs::write(
                &network_list_path,
                toml::to_string_pretty(&self.network_list)?,
            )?;
        }

        Ok(())
    }

    fn setup_ssl(&mut self) -> Result<Connector> {
        let ssl_dir = self.path.join("ssl");
        if !ssl_dir.try_exists()? {
            fs::create_dir_all(&ssl_dir)?;
        }

        let cert = load_ssl_cert(
            ssl_dir
                .join("wallet.crt")
                .to_str()
                .expect("invalid crt file name"),
            ssl_dir
                .join("wallet.key")
                .to_str()
                .expect("invalid key file name"),
        )?;

        Ok(create_rustls_connector(&cert)?)
    }

    fn setup_sync_manager(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        let connector = self.setup_ssl()?;

        let (sync_manager, command_sender, receiver) = SyncManager::new(
            SyncOptions {
                target_peers: self.config.network.target_peers.try_into()?,
                discover_peers: self.config.network.discover_peers,
                max_peer_age_seconds: 3600 * 8,
                dns_batch_size: 10,
                connection_batch_size: 30,
                delta_sync: self
                    .wallet_config()
                    .cloned()
                    .unwrap_or_default()
                    .delta_sync(&self.wallet_config.defaults),
                puzzle_batch_size_per_peer: 5,
                timeouts: Timeouts::default(),
                testing: false,
            },
            self.peer_state.clone(),
            self.wallet.clone(),
            self.network().clone(),
            connector,
        );

        tokio::spawn(sync_manager.sync());
        self.command_sender = command_sender;

        // Create a broadcast channel to split events between Tauri app and webhook consumer
        let (tx, _rx) = tokio::sync::broadcast::channel(100);

        // Create a receiver for the Tauri app before we move tx
        let tauri_receiver = tx.subscribe();

        // Spawn a task that forwards events from the sync manager to the broadcast channel
        let webhook_manager = self.webhook_manager.clone();
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(event) = receiver.recv().await {
                // Send to broadcast channel (for Tauri app)
                let _ = tx.send(event.clone());

                // Also handle for webhooks directly
                Self::handle_sync_event_for_webhooks(&webhook_manager, event).await;
            }
        });

        // Convert broadcast receiver to mpsc receiver for compatibility
        let (tauri_tx, tauri_rx) = mpsc::channel(100);

        // Give webhook manager access to the event sender for internal webhook events
        let webhook_manager_for_sender = self.webhook_manager.clone();
        let event_sender = tauri_tx.clone();
        tokio::spawn(async move {
            webhook_manager_for_sender
                .set_event_sender(event_sender)
                .await;
        });

        tokio::spawn(async move {
            let mut tauri_receiver = tauri_receiver;
            while let Ok(event) = tauri_receiver.recv().await {
                let _ = tauri_tx.send(event).await;
            }
        });

        Ok(tauri_rx)
    }

    async fn handle_sync_event_for_webhooks(webhook_manager: &WebhookManager, event: SyncEvent) {
        // Convert wallet SyncEvent to webhook payload
        // Skip internal webhook events - they should not be sent over webhooks
        let (event_type, data) = match event {
            SyncEvent::Start(ip) => (
                "start",
                serde_json::json!({
                    "ip": ip.to_string()
                }),
            ),
            SyncEvent::Stop => ("stop", serde_json::json!({})),
            SyncEvent::Subscribed => ("subscribed", serde_json::json!({})),
            SyncEvent::DerivationIndex { next_index } => (
                "derivation",
                serde_json::json!({
                    "next_index": next_index
                }),
            ),
            SyncEvent::TransactionUpdated { transaction_id } => (
                "transaction_updated",
                serde_json::json!({
                    "transaction_id": transaction_id.to_string()
                }),
            ),
            SyncEvent::TransactionConfirmed { transaction_id } => (
                "transaction_confirmed",
                serde_json::json!({
                    "transaction_id": transaction_id.to_string()
                }),
            ),
            SyncEvent::TransactionFailed {
                transaction_id,
                error,
            } => (
                "transaction_failed",
                serde_json::json!({
                    "transaction_id": transaction_id.to_string(),
                    "error": error
                }),
            ),
            SyncEvent::OfferUpdated { offer_id, status } => (
                "offer_updated",
                serde_json::json!({
                    "offer_id": offer_id.to_string(),
                    "status": format!("{:?}", status)
                }),
            ),
            SyncEvent::CoinsUpdated { coin_ids } => {
                if coin_ids.is_empty() {
                    return;
                }
                (
                    "coins_updated",
                    serde_json::json!({
                        "coin_ids": coin_ids.iter().map(ToString::to_string).collect::<Vec<_>>()
                    }),
                )
            }
            SyncEvent::PuzzleBatchSynced => ("puzzle_batch_synced", serde_json::json!({})),
            SyncEvent::CatInfo { asset_ids } => (
                "cat_info",
                serde_json::json!({
                    "asset_ids": asset_ids.iter().map(ToString::to_string).collect::<Vec<_>>()
                }),
            ),
            SyncEvent::DidInfo { launcher_id } => (
                "did_info",
                serde_json::json!({
                    "launcher_id": launcher_id.to_string()
                }),
            ),
            SyncEvent::NftData { launcher_ids } => (
                "nft_data",
                serde_json::json!({
                    "launcher_ids": launcher_ids.iter().map(ToString::to_string).collect::<Vec<_>>()
                }),
            ),
            // Internal webhook notifications - do not send over webhooks
            SyncEvent::WebhooksChanged | SyncEvent::WebhookInvoked => return,
        };

        webhook_manager
            .send_event(event_type.to_string(), data)
            .await;
    }

    pub async fn switch_network(&mut self) -> Result<()> {
        self.webhook_manager.set_network(self.network_id()).await;

        self.command_sender
            .send(SyncCommand::SwitchNetwork(self.network().clone()))
            .await?;

        Ok(())
    }

    pub async fn switch_wallet(&mut self) -> Result<()> {
        self.switch_network().await?;

        let Some(fingerprint) = self.config.global.fingerprint else {
            self.wallet = None;
            self.webhook_manager.set_fingerprint(None).await;

            self.command_sender
                .send(SyncCommand::SwitchWallet {
                    wallet: None,
                    delta_sync: self.wallet_config.defaults.delta_sync,
                })
                .await?;

            return Ok(());
        };

        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Err(Error::UnknownFingerprint);
        };

        let intermediate_pk = master_to_wallet_unhardened_intermediate(&master_pk);

        let pool = self.connect_to_database(fingerprint).await?;
        let db = Database::new(pool);

        db.run_rust_migrations(self.network().ticker.clone())
            .await?;

        let wallet_config = self.wallet_config().cloned().unwrap_or_default();

        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            self.network().genesis_challenge,
            AggSigConstants::new(self.network().agg_sig_me()),
            wallet_config
                .change_address
                .as_ref()
                .map(|address| Address::decode(address))
                .transpose()?
                .map(|address| address.puzzle_hash),
        ));

        self.wallet = Some(wallet.clone());
        self.unit = Unit {
            ticker: self.network().ticker.clone(),
            precision: self.network().precision,
        };
        self.webhook_manager
            .set_fingerprint(Some(fingerprint))
            .await;

        self.command_sender
            .send(SyncCommand::SwitchWallet {
                wallet: Some(wallet),
                delta_sync: wallet_config.delta_sync(&self.wallet_config.defaults),
            })
            .await?;

        Ok(())
    }

    pub async fn setup_peers(&mut self) -> Result<()> {
        let peer_dir = self.path.join("peers");

        if !peer_dir.exists() {
            fs::create_dir_all(&peer_dir)?;
        }

        let peer_path = peer_dir.join(format!("{}.bin", self.network_id()));

        let peers = if peer_path.try_exists()? {
            Peers::from_bytes(&fs::read(&peer_path)?).unwrap_or_else(|error| {
                error!("Failed to load peers, reverting to default: {error}");
                Peers::default()
            })
        } else {
            Peers::default()
        };

        let mut state = self.peer_state.lock().await;

        for (&ip, &timestamp) in &peers.banned {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before epoch")
                .as_secs();

            if now >= timestamp {
                continue;
            }

            state.ban(ip, Duration::from_secs(timestamp - now), "already banned");
        }

        for &ip in &peers.connections {
            if state.peer(ip).is_some() {
                continue;
            }

            self.command_sender
                .send(SyncCommand::ConnectPeer {
                    ip,
                    user_managed: false,
                })
                .await?;
        }

        for &ip in &peers.user_managed {
            if state.peer(ip).is_some() {
                continue;
            }

            self.command_sender
                .send(SyncCommand::ConnectPeer {
                    ip,
                    user_managed: true,
                })
                .await?;
        }

        Ok(())
    }

    pub async fn save_peers(&self) -> Result<()> {
        let peer_dir = self.path.join("peers");

        if !peer_dir.exists() {
            fs::create_dir_all(&peer_dir)?;
        }

        let mut peers = Peers::default();
        let mut state = self.peer_state.lock().await;

        for peer in state.user_managed_peers() {
            peers.user_managed.insert(peer.socket_addr().ip());
        }

        for peer in state.auto_discovered_peers() {
            peers.connections.insert(peer.socket_addr().ip());
        }

        for (&ip, &ban) in state.banned_peers() {
            peers.banned.insert(ip, ban);
        }

        let peer_path = peer_dir.join(format!("{}.bin", self.network_id()));
        fs::write(&peer_path, peers.to_bytes()?)?;

        Ok(())
    }

    pub fn parse_address(&self, input: String) -> Result<Bytes32> {
        let address = Address::decode(&input)?;

        if address.prefix != self.network().prefix() {
            return Err(Error::AddressPrefix(address.prefix));
        }

        Ok(address.puzzle_hash)
    }

    pub async fn connect_to_database(&self, fingerprint: u32) -> Result<SqlitePool> {
        let path = self.wallet_db_path(fingerprint)?;

        let pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path.display()))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace)
                    .synchronous(SqliteSynchronous::Normal)
                    .busy_timeout(Duration::from_secs(60)),
            )
            .await?;

        if let Err(_error) = sqlx::migrate!("../../migrations").run(&pool).await {
            return Err(Error::DatabaseVersionTooOld);
        }

        Ok(pool)
    }

    pub fn wallet_db_path(&self, fingerprint: u32) -> Result<PathBuf> {
        let path = self.path.join("wallets").join(fingerprint.to_string());
        fs::create_dir_all(&path)?;
        let network_id = self
            .wallet_config
            .wallets
            .iter()
            .find_map(|wallet| {
                if wallet.fingerprint == fingerprint {
                    wallet.network.clone()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| self.network_id());
        let path = path.join(format!("{network_id}.sqlite"));
        Ok(path)
    }

    pub fn wallet_config(&self) -> Option<&sage_config::Wallet> {
        self.config.global.fingerprint.and_then(|fingerprint| {
            self.wallet_config
                .wallets
                .iter()
                .find(|w| w.fingerprint == fingerprint)
        })
    }

    pub fn network(&self) -> &Network {
        if let Some(wallet) = self.wallet_config() {
            if let Some(network) = &wallet.network {
                return self
                    .network_list
                    .by_name(network)
                    .expect("network not found");
            }
        }

        self.network_list
            .by_name(&self.config.network.default_network)
            .expect("network not found")
    }

    pub fn network_id(&self) -> String {
        self.network().network_id()
    }

    pub fn wallet(&self) -> Result<Arc<Wallet>> {
        let Some(fingerprint) = self.config.global.fingerprint else {
            return Err(Error::NotLoggedIn);
        };

        if !self.keychain.contains(fingerprint) {
            return Err(Error::UnknownFingerprint);
        }

        let wallet = self.wallet.as_ref().ok_or(Error::NotLoggedIn)?;

        if wallet.fingerprint != fingerprint {
            return Err(Error::NotLoggedIn);
        }

        Ok(wallet.clone())
    }

    pub fn save_config(&self) -> Result<()> {
        let config = toml::to_string_pretty(&self.config)?;
        fs::write(self.path.join("config.toml"), config)?;
        let wallet_config = toml::to_string_pretty(&self.wallet_config)?;
        fs::write(self.path.join("wallets.toml"), wallet_config)?;
        let network_list = toml::to_string_pretty(&self.network_list)?;
        fs::write(self.path.join("networks.toml"), network_list)?;
        Ok(())
    }

    pub fn save_keychain(&self) -> Result<()> {
        fs::write(self.path.join("keys.bin"), self.keychain.to_bytes()?)?;
        Ok(())
    }

    pub async fn save_webhooks_config(&mut self) -> Result<()> {
        use sage_config::WebhookEntry;

        let entries = self.webhook_manager.get_webhook_entries().await;
        self.config.webhooks.webhooks = entries
            .into_iter()
            .map(
                |(
                    id,
                    url,
                    events,
                    enabled,
                    secret,
                    last_delivered_at,
                    last_delivery_attempt_at,
                    consecutive_failures,
                )| {
                    WebhookEntry {
                        id,
                        url,
                        events,
                        enabled,
                        secret,
                        last_delivered_at,
                        last_delivery_attempt_at,
                        consecutive_failures,
                    }
                },
            )
            .collect();

        self.save_config()?;
        Ok(())
    }

    async fn setup_webhooks(&mut self) -> Result<()> {
        use crate::webhook_manager::WebhookEntryTuple;

        let entries: Vec<WebhookEntryTuple> = self
            .config
            .webhooks
            .webhooks
            .iter()
            .map(|w| {
                (
                    w.id.clone(),
                    w.url.clone(),
                    w.events.clone(),
                    w.enabled,
                    w.secret.clone(),
                    w.last_delivered_at,
                    w.last_delivery_attempt_at,
                    w.consecutive_failures,
                )
            })
            .collect();

        if !entries.is_empty() {
            self.webhook_manager.load_webhooks(entries).await;
            info!(
                "Loaded {} webhooks from config",
                self.config.webhooks.webhooks.len()
            );
        }

        Ok(())
    }
}
