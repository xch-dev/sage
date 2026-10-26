use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, MutexGuard, Once},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{
    chia::bls::master_to_wallet_unhardened_intermediate,
    client::{Connector, create_rustls_connector, load_ssl_cert},
    prelude::*,
};
use indexmap::IndexMap;
use sage_api::{Unit, XCH};
use sage_config::{
    Config, Network, NetworkConfig, NetworkList, OldConfig, OldNetwork, Wallet as WalletEntry,
    WalletConfig, is_old_config, migrate_config, migrate_networks,
};
use sage_database::Database;
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts, Wallet};
use serde::de::DeserializeOwned;
use sqlx::{
    ConnectOptions, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tracing::{Level, debug, error, info, warn};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer, Registry, filter::filter_fn, fmt, layer::SubscriberExt,
};

use crate::{DataDirLock, Error, FileLock, Result, peers::Peers};

static LOGGING_SETUP: Once = Once::new();

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
    pub test: bool,
    /// Shared between requests so that connections are reused.
    pub http_client: reqwest::Client,
    data_dir_lock: Option<DataDirLock>,
    writable: bool,
}

impl Sage {
    pub fn new(path: &Path, test: bool) -> Self {
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
            test,
            http_client: reqwest::Client::new(),
            data_dir_lock: None,
            writable: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        fs::create_dir_all(&self.path)?;

        self.lock_data_dir()?;

        let result = self.initialize_locked().await;

        // If initialization fails partway, the in-memory state may be incomplete (for example,
        // an empty wallet list), so it must never be saved over the files.
        if let Err(error) = &result {
            error!("Failed to initialize Sage: {error}");
            self.writable = false;
        }

        result
    }

    /// This must happen before anything is read, so that the state can't be stale.
    /// The lock is kept even if initialization fails, until the process exits.
    fn lock_data_dir(&mut self) -> Result<()> {
        self.data_dir_lock = Some(DataDirLock::acquire(&self.path)?);
        self.writable = true;
        Ok(())
    }

    async fn initialize_locked(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        self.load_data()?;

        let (receiver, sync_task) = self.setup_sync_manager()?;

        // Don't leave the sync manager running if initialization fails.
        if let Err(error) = self.setup_peers().await {
            sync_task.abort();
            self.command_sender = mpsc::channel(1).0;
            return Err(error);
        }

        info!("Sage wallet initialized");

        Ok(receiver)
    }

    /// Loads the config files and keychain, repairing them where that's safe to do.
    fn load_data(&mut self) -> Result<()> {
        // Logging depends on the config, so problems found while loading it are logged afterward.
        // If the config can't be loaded, logging is still set up with the default log level.
        let mut warnings = Vec::new();
        let config_result = self.setup_config(&mut warnings);
        self.setup_logging(&mut warnings)?;

        for warning in warnings {
            warn!("{warning}");
        }

        config_result?;
        remove_temp_files(&self.path);

        if let Some(lock) = &self.data_dir_lock
            && let FileLock::Unsupported(error) = lock.file_lock()
        {
            warn!("File locking is not supported for the data directory: {error}");
        }

        self.setup_keys()?;
        self.setup_wallet_config()?;
        self.fix_network_references()?;
        self.recover_missing_wallets()?;

        Ok(())
    }

    fn setup_logging(&mut self, warnings: &mut Vec<String>) -> Result<()> {
        // An invalid log level shouldn't prevent startup, or hide the reason for other failures.
        let log_level: Level = self.config.global.log_level.parse().unwrap_or_else(|_| {
            warnings.push(format!(
                "Invalid log level {}, using INFO instead",
                self.config.global.log_level
            ));
            Level::INFO
        });

        // Common filter string
        let filter_string = format!("{log_level},rustls=off,tungstenite=off,h2=off,hyper=off");

        let file_layer = if self.test {
            None
        } else {
            let log_dir = self.path.join("log");

            if !log_dir.exists() {
                std::fs::create_dir_all(&log_dir)?;
            }

            // Create rotated log file
            let log_file = Builder::new()
                .filename_prefix("app.log")
                .rotation(Rotation::DAILY)
                .max_log_files(3)
                .build(log_dir)?;

            // File layer - always without ANSI
            Some(
                fmt::layer()
                    .with_target(false)
                    .with_ansi(false)
                    .with_writer(log_file)
                    .compact()
                    .with_filter(EnvFilter::new(&filter_string)),
            )
        };

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
        LOGGING_SETUP.call_once(|| {
            if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
                error!("Failed to set global default logger: {error}");
            }
        });

        Ok(())
    }

    fn setup_keys(&mut self) -> Result<()> {
        let key_path = self.path.join("keys.bin");

        if key_path.try_exists()? {
            let data = fs::read(&key_path)?;
            self.keychain = Keychain::from_bytes(&data)?;
        } else {
            write_atomic(&key_path, &self.keychain.to_bytes()?)?;
        }

        Ok(())
    }

    /// Adds a wallet entry for every key in the keychain that isn't listed in `wallets.toml`.
    /// Otherwise those keys are inaccessible, since the wallet list is driven by the config.
    fn recover_missing_wallets(&mut self) -> Result<()> {
        let mut missing: Vec<u32> = self
            .keychain
            .fingerprints()
            .filter(|fingerprint| {
                !self
                    .wallet_config
                    .wallets
                    .iter()
                    .any(|wallet| wallet.fingerprint == *fingerprint)
            })
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        missing.sort_unstable();

        for fingerprint in missing {
            warn!("Recovering wallet {fingerprint}, which is in the keychain but not the config");

            self.wallet_config.wallets.push(WalletEntry {
                fingerprint,
                ..Default::default()
            });
        }

        self.save_config()
    }

    fn setup_config(&mut self, warnings: &mut Vec<String>) -> Result<()> {
        let config_path = self.path.join("config.toml");
        let wallet_config_path = self.path.join("wallets.toml");
        let network_list_path = self.path.join("networks.toml");

        match read_config_file(&config_path, warnings)? {
            Some(text) if is_old_config(&text) => {
                // The migrated wallet config is loaded from the file later, in `setup_wallet_config`.
                let old_config: OldConfig = parse_config_file(&config_path, &text)?;
                let (config, wallet_config) = migrate_config(old_config)?;
                self.config = config;
                write_atomic(&config_path, toml::to_string_pretty(&self.config)?)?;
                write_atomic(&wallet_config_path, toml::to_string_pretty(&wallet_config)?)?;
            }
            Some(text) => self.config = parse_config_file(&config_path, &text)?,
            None => write_atomic(&config_path, toml::to_string_pretty(&self.config)?)?,
        }

        match read_config_file(&network_list_path, warnings)? {
            Some(text) => {
                // An empty file isn't an old network list, since migrating it would remove every network.
                if let Some(old_network_list) =
                    toml::from_str::<IndexMap<String, OldNetwork>>(&text)
                        .ok()
                        .filter(|networks| !networks.is_empty())
                {
                    self.network_list = migrate_networks(old_network_list);
                    write_atomic(
                        &network_list_path,
                        toml::to_string_pretty(&self.network_list)?,
                    )?;
                } else {
                    self.network_list = parse_config_file(&network_list_path, &text)?;
                }
            }
            None => write_atomic(
                &network_list_path,
                toml::to_string_pretty(&self.network_list)?,
            )?,
        }

        Ok(())
    }

    /// Loads the wallet config. If it's missing or damaged, this falls back to an empty one, and
    /// the wallets are then recovered from the keychain.
    fn setup_wallet_config(&mut self) -> Result<()> {
        let path = self.path.join("wallets.toml");
        let mut warnings = Vec::new();

        let text = read_config_file(&path, &mut warnings)?;

        for warning in warnings {
            warn!("{warning}");
        }

        if let Some(text) = text {
            self.wallet_config = parse_config_file(&path, &text)?;
        } else {
            self.wallet_config = WalletConfig::default();
            write_atomic(&path, toml::to_string_pretty(&self.wallet_config)?)?;
        }

        Ok(())
    }

    /// Resets references to networks that don't exist, which would otherwise cause a panic when
    /// looking up the current network. This can happen if the network list was reset or edited.
    fn fix_network_references(&mut self) -> Result<()> {
        let mut changed = false;

        if self.network_list.networks.is_empty() {
            warn!("Network list is empty, resetting it to the default networks");
            self.network_list = NetworkList::default();
            changed = true;
        }

        if self
            .network_list
            .by_name(&self.config.network.default_network)
            .is_none()
        {
            let fallback = NetworkConfig::default().default_network;

            let fallback = if self.network_list.by_name(&fallback).is_some() {
                fallback
            } else {
                self.network_list.networks[0].name.clone()
            };

            warn!(
                "Network {} doesn't exist, switching to {fallback}",
                self.config.network.default_network
            );

            self.config.network.default_network = fallback;
            changed = true;
        }

        for wallet in &mut self.wallet_config.wallets {
            if let Some(network) = &wallet.network
                && self.network_list.by_name(network).is_none()
            {
                warn!(
                    "Network {network} for wallet {} doesn't exist, using the default network",
                    wallet.fingerprint
                );
                wallet.network = None;
                changed = true;
            }
        }

        if changed {
            self.save_config()?;
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

    fn setup_sync_manager(&mut self) -> Result<(mpsc::Receiver<SyncEvent>, JoinHandle<()>)> {
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
                timeouts: if self.test {
                    Timeouts {
                        sync_delay: Duration::from_millis(100),
                        nft_uri_delay: Duration::from_millis(100),
                        cat_delay: Duration::from_millis(100),
                        puzzle_delay: Duration::from_millis(100),
                        transaction_delay: Duration::from_millis(100),
                        offer_delay: Duration::from_millis(100),
                        ..Default::default()
                    }
                } else {
                    Timeouts::default()
                },
                testing: self.test,
            },
            self.peer_state.clone(),
            self.wallet.clone(),
            self.network().clone(),
            connector,
        );

        let sync_task = tokio::spawn(sync_manager.sync());
        self.command_sender = command_sender;

        Ok((receiver, sync_task))
    }

    pub async fn switch_network(&mut self) -> Result<()> {
        self.command_sender
            .send(SyncCommand::SwitchNetwork(Box::new(self.network().clone())))
            .await?;

        Ok(())
    }

    pub async fn switch_wallet(&mut self) -> Result<()> {
        self.switch_network().await?;

        let Some(fingerprint) = self.config.global.fingerprint else {
            self.wallet = None;

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

        // Peers are only a cache, and are reset if the file is damaged, so this doesn't need the
        // disk syncs of an atomic write. It runs every few seconds.
        let peer_path = peer_dir.join(format!("{}.bin", self.network_id()));
        let _guard = self.write_guard()?;
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
        self.connect_to_pool(path).await
    }

    pub async fn connect_to_pool(&self, path: PathBuf) -> Result<SqlitePool> {
        let pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path.display()))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace)
                    .synchronous(SqliteSynchronous::Normal)
                    .busy_timeout(Duration::from_mins(1)),
            )
            .await?;

        if let Err(_error) = sqlx::migrate!("../../migrations").run(&pool).await {
            return Err(Error::DatabaseVersionTooOld);
        }

        update_query_planner_stats(&pool).await;

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

    /// Falls back to the default network if the wallet's network doesn't exist, and then to the
    /// first network. The network list is never empty, since that's corrected during startup.
    pub fn network(&self) -> &Network {
        self.wallet_config()
            .and_then(|wallet| wallet.network.as_deref())
            .and_then(|network| self.network_list.by_name(network))
            .or_else(|| {
                self.network_list
                    .by_name(&self.config.network.default_network)
            })
            .or_else(|| self.network_list.networks.first())
            .expect("network list is empty")
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

    /// Files in the data directory may only be written while this instance holds the lock.
    /// The returned guard must be held for the duration of the write.
    fn write_guard(&self) -> Result<MutexGuard<'_, u64>> {
        if !self.writable {
            return Err(Error::NotInitialized);
        }

        // If there's no guard, another instance in this process has taken over the data directory.
        self.data_dir_lock
            .as_ref()
            .and_then(DataDirLock::write_guard)
            .ok_or(Error::AlreadyRunning)
    }

    pub fn save_config(&self) -> Result<()> {
        let _guard = self.write_guard()?;

        let config = toml::to_string_pretty(&self.config)?;
        write_if_changed(&self.path.join("config.toml"), config)?;
        let wallet_config = toml::to_string_pretty(&self.wallet_config)?;
        write_if_changed(&self.path.join("wallets.toml"), wallet_config)?;
        let network_list = toml::to_string_pretty(&self.network_list)?;
        write_if_changed(&self.path.join("networks.toml"), network_list)?;
        Ok(())
    }

    pub fn save_keychain(&self) -> Result<()> {
        let _guard = self.write_guard()?;

        write_atomic(&self.path.join("keys.bin"), &self.keychain.to_bytes()?)?;
        Ok(())
    }
}

/// Refreshes the query planner's table statistics.
async fn update_query_planner_stats(pool: &SqlitePool) {
    if let Err(error) = Database::new(pool.clone()).analyze().await {
        warn!("Failed to update query planner stats: {error}");
    } else {
        info!("Updated database query planner statistics");
    }
}

/// Reads a config file, returning `None` if it doesn't exist. Interrupted writes can leave zeroed
/// blocks behind, so a file like that is moved aside and also treated as missing. Other problems
/// may be from hand edits that the user can fix, so those are reported instead.
fn read_config_file(path: &Path, warnings: &mut Vec<String>) -> Result<Option<String>> {
    if !path.try_exists()? {
        return Ok(None);
    }

    let data = fs::read(path)?;

    // UTF-16 text contains zero bytes too, but comes from an editor rather than a failed write.
    let is_utf16 = data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]);

    if !data.contains(&0) || is_utf16 {
        return String::from_utf8(data)
            .map(Some)
            .map_err(|_| Error::InvalidConfig {
                path: path.display().to_string(),
                error: "The file is not UTF-8 encoded".to_string(),
            });
    }

    // Keep the damaged file around, since it may still contain some of the settings.
    let path = resolve_symlinks(path)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut backup_path = path.with_extension(format!("toml.damaged-{timestamp}"));
    let mut suffix = 1;

    while backup_path.try_exists()? {
        backup_path = path.with_extension(format!("toml.damaged-{timestamp}-{suffix}"));
        suffix += 1;
    }

    fs::rename(&path, &backup_path)?;

    warnings.push(format!(
        "{} was damaged, so it was moved to {} and replaced with defaults",
        path.display(),
        backup_path.display()
    ));

    Ok(None)
}

fn parse_config_file<T: DeserializeOwned>(path: &Path, text: &str) -> Result<T> {
    toml::from_str(text).map_err(|error| Error::InvalidConfig {
        path: path.display().to_string(),
        error: error.to_string(),
    })
}

const TEMP_FILE_PREFIX: &str = ".sage-write-";

/// Removes temporary files left behind by `write_atomic` if the process exited mid-write.
/// This is only safe while holding the data directory lock, since otherwise they may be in use.
fn remove_temp_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(std::result::Result::ok) {
        let is_temp_file = entry
            .file_name()
            .to_string_lossy()
            .starts_with(TEMP_FILE_PREFIX)
            && entry.file_type().is_ok_and(|file_type| file_type.is_file());

        if is_temp_file && let Err(error) = fs::remove_file(entry.path()) {
            warn!(
                "Failed to remove temporary file {}: {error}",
                entry.path().display()
            );
        }
    }
}

/// Skips the write if the file already has these contents, to avoid unnecessary disk syncs.
fn write_if_changed(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    if fs::read(path).is_ok_and(|existing| existing == contents.as_ref()) {
        return Ok(());
    }

    write_atomic(path, contents)
}

/// Follows symlinks to the file they point to, so that writing replaces that file rather than the
/// symlink itself. Unlike `fs::canonicalize`, the final target doesn't need to exist yet.
fn resolve_symlinks(path: &Path) -> Result<PathBuf> {
    let mut path = path.to_path_buf();

    // The limit guards against symlink loops.
    for _ in 0..32 {
        if !fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_symlink()) {
            return Ok(path);
        }

        let target = fs::read_link(&path)?;

        path = match path.parent() {
            Some(parent) => parent.join(target),
            None => target,
        };
    }

    Err(io::Error::other(format!("Too many levels of symlinks at {}", path.display())).into())
}

/// Writes to a temporary file in the same directory, then renames it over the destination.
/// A crash or forced shutdown mid-write can't leave the destination truncated or empty.
fn write_atomic(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = resolve_symlinks(path)?;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut file = tempfile::Builder::new()
        .prefix(TEMP_FILE_PREFIX)
        .tempfile_in(dir)?;
    file.write_all(contents.as_ref())?;

    // Keep the existing file's permissions, rather than those of the temporary file.
    if let Ok(metadata) = fs::metadata(&path) {
        file.as_file().set_permissions(metadata.permissions())?;
    }

    file.as_file().sync_all()?;

    // On Windows, replacing a file fails while another program (such as an antivirus scanner)
    // briefly has it open, so retry a few times before giving up. This blocks for at most 150ms,
    // and only in that case.
    let mut attempt = 0;

    while let Err(error) = file.persist(&path) {
        attempt += 1;

        if !cfg!(windows) || attempt >= 4 || error.error.kind() != io::ErrorKind::PermissionDenied {
            return Err(error.error.into());
        }

        file = error.file;
        std::thread::sleep(Duration::from_millis(25 * attempt));
    }

    // Sync the directory so that the rename itself survives a power loss.
    #[cfg(unix)]
    if let Err(error) = fs::File::open(dir).and_then(|dir| dir.sync_all()) {
        debug!("Failed to sync directory {}: {error}", dir.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn load(path: &Path) -> Result<Sage> {
        let mut sage = Sage::new(path, true);
        sage.lock_data_dir()?;
        sage.load_data()?;
        Ok(sage)
    }

    fn add_key(sage: &mut Sage, seed: u8) -> Result<u32> {
        let master_pk = SecretKey::from_seed(&[seed; 32]).public_key();
        Ok(sage.keychain.add_public_key(&master_pk)?)
    }

    #[test]
    fn test_recover_wallets_missing_from_config() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let named = add_key(&mut sage, 1)?;
        let lost = add_key(&mut sage, 2)?;
        sage.wallet_config.wallets.push(WalletEntry {
            name: "Named".to_string(),
            fingerprint: named,
            ..Default::default()
        });
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        // Simulate a wallet config that lost an entry
        fs::write(
            dir.path().join("wallets.toml"),
            format!("[[wallets]]\nname = \"Named\"\nfingerprint = {named}\n"),
        )?;

        let sage = load(dir.path())?;
        let wallets = &sage.wallet_config.wallets;
        assert_eq!(wallets.len(), 2);
        assert_eq!(wallets[0].name, "Named");
        assert_eq!(wallets[1].fingerprint, lost);
        drop(sage);

        // The recovered entry is persisted
        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 2);

        Ok(())
    }

    #[test]
    fn test_save_requires_lock() -> Result<()> {
        let dir = TempDir::new()?;

        let sage = Sage::new(dir.path(), true);
        assert!(matches!(sage.save_config(), Err(Error::NotInitialized)));
        assert!(matches!(sage.save_keychain(), Err(Error::NotInitialized)));
        assert!(!dir.path().join("wallets.toml").try_exists()?);

        Ok(())
    }

    #[test]
    fn test_recover_wallets_when_config_deleted() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        fs::remove_file(dir.path().join("wallets.toml"))?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].fingerprint, fingerprint);

        Ok(())
    }

    #[test]
    fn test_wallet_config_kept_when_config_deleted() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.wallet_config.wallets.push(WalletEntry {
            name: "Named".to_string(),
            fingerprint,
            ..Default::default()
        });
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        fs::remove_file(dir.path().join("config.toml"))?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].name, "Named");

        Ok(())
    }

    #[test]
    fn test_damaged_wallet_config_is_backed_up() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        // Simulate a partially written file
        let corrupt = b"[[wallets]]\nname = \"Na\0\0\0";
        fs::write(dir.path().join("wallets.toml"), corrupt)?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].fingerprint, fingerprint);

        let backups: Vec<_> = fs::read_dir(dir.path())?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("wallets.toml.damaged-")
            })
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path())?, corrupt);

        Ok(())
    }

    #[test]
    fn test_invalid_wallet_config_is_reported() -> Result<()> {
        let dir = TempDir::new()?;

        drop(load(dir.path())?);

        // A hand edit with a typo shouldn't silently reset the wallet names and settings
        let invalid = "[[wallets]]\nname = Main\n";
        fs::write(dir.path().join("wallets.toml"), invalid)?;

        assert!(matches!(load(dir.path()), Err(Error::InvalidConfig { .. })));
        assert_eq!(
            fs::read_to_string(dir.path().join("wallets.toml"))?,
            invalid
        );

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_write_atomic_follows_dangling_symlink() -> Result<()> {
        use std::os::unix::fs::symlink;

        let dir = TempDir::new()?;
        let target = dir.path().join("target.toml");
        let link = dir.path().join("link.toml");

        symlink("target.toml", &link)?;

        write_atomic(&link, "new")?;

        assert!(fs::symlink_metadata(&link)?.is_symlink());
        assert_eq!(fs::read_to_string(&target)?, "new");

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_write_atomic_preserves_symlink_and_permissions() -> Result<()> {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let dir = TempDir::new()?;
        let target = dir.path().join("target.toml");
        let link = dir.path().join("link.toml");

        fs::write(&target, "old")?;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640))?;
        symlink(&target, &link)?;

        write_atomic(&link, "new")?;

        assert!(fs::symlink_metadata(&link)?.is_symlink());
        assert_eq!(fs::read_to_string(&target)?, "new");
        assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o777, 0o640);

        Ok(())
    }

    #[test]
    fn test_empty_config_is_not_migrated() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.wallet_config.wallets.push(WalletEntry {
            name: "Named".to_string(),
            fingerprint,
            ..Default::default()
        });
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        // An interrupted write can leave the config empty
        fs::write(dir.path().join("config.toml"), "")?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].name, "Named");

        Ok(())
    }

    #[test]
    fn test_old_config_is_migrated() -> Result<()> {
        let dir = TempDir::new()?;

        fs::write(
            dir.path().join("config.toml"),
            "[app]\nlog_level = \"INFO\"\n\n[wallets.1000000]\nname = \"Old\"\n",
        )?;

        let sage = load(dir.path())?;
        assert_eq!(sage.config.version, 2);
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].name, "Old");
        assert_eq!(sage.wallet_config.wallets[0].fingerprint, 1_000_000);

        Ok(())
    }

    #[test]
    fn test_empty_or_damaged_network_list_is_reset() -> Result<()> {
        for contents in [&b""[..], b"\0\0\0\0"] {
            let dir = TempDir::new()?;

            drop(load(dir.path())?);
            fs::write(dir.path().join("networks.toml"), contents)?;

            let sage = load(dir.path())?;
            assert_eq!(sage.network().name, "mainnet");
        }

        Ok(())
    }

    #[test]
    fn test_missing_network_references_are_reset() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.wallet_config.wallets.push(WalletEntry {
            fingerprint,
            network: Some("removed".to_string()),
            ..Default::default()
        });
        sage.config.network.default_network = "removed".to_string();
        sage.config.global.fingerprint = Some(fingerprint);
        sage.save_keychain()?;
        sage.save_config()?;
        drop(sage);

        let sage = load(dir.path())?;
        assert_eq!(sage.config.network.default_network, "mainnet");
        assert_eq!(sage.wallet_config.wallets[0].network, None);
        assert_eq!(sage.network().name, "mainnet");

        Ok(())
    }

    #[test]
    fn test_newer_instance_takes_over_writes() -> Result<()> {
        let dir = TempDir::new()?;

        let old = load(dir.path())?;
        let new = load(dir.path())?;

        assert!(matches!(old.save_config(), Err(Error::AlreadyRunning)));
        new.save_config()?;

        Ok(())
    }

    #[test]
    fn test_leftover_temp_files_are_removed() -> Result<()> {
        let dir = TempDir::new()?;

        fs::write(dir.path().join(".sage-write-AbC123"), "partial")?;
        fs::write(dir.path().join(".tmpUnrelated"), "keep")?;

        drop(load(dir.path())?);

        assert!(!dir.path().join(".sage-write-AbC123").try_exists()?);
        assert!(dir.path().join(".tmpUnrelated").try_exists()?);

        Ok(())
    }

    #[test]
    fn test_non_utf8_config_is_reported() -> Result<()> {
        let dir = TempDir::new()?;

        drop(load(dir.path())?);

        // "Café" saved by an editor in Windows-1252 rather than UTF-8
        let contents = b"[[wallets]]\nname = \"Caf\xE9\"\nfingerprint = 1\n";
        fs::write(dir.path().join("wallets.toml"), contents)?;

        assert!(matches!(load(dir.path()), Err(Error::InvalidConfig { .. })));
        assert_eq!(fs::read(dir.path().join("wallets.toml"))?, contents);

        Ok(())
    }
}
