use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Once},
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
    Config, Network, NetworkList, OldConfig, OldNetwork, Wallet as WalletEntry, WalletConfig,
    is_old_config, migrate_config, migrate_networks,
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
use tracing::{Level, error, info, warn};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer, Registry, filter::filter_fn, fmt, layer::SubscriberExt,
};

use crate::{DataDirLock, Error, Result, peers::Peers};

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
    /// Whether the config and keychain can be saved. This is only the case once initialization
    /// has started, and stops if it fails, since the state in memory may then be incomplete.
    writable: bool,
    data_dir_lock: Option<DataDirLock>,
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
            writable: false,
            data_dir_lock: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        self.writable = true;

        let result = self.try_initialize().await;

        if let Err(error) = &result {
            error!("Failed to initialize Sage: {error}");
            self.writable = false;
        }

        result
    }

    async fn try_initialize(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        fs::create_dir_all(&self.path)?;

        // This must happen before anything is read, so that the state can't be stale.
        self.data_dir_lock = Some(DataDirLock::acquire(&self.path)?);

        self.setup_keys()?;
        self.setup_config()?;
        self.setup_logging()?;

        if let Some(DataDirLock::Unsupported(error)) = &self.data_dir_lock {
            warn!("File locking is not supported for the data directory: {error}");
        }
        self.recover_missing_wallets()?;

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

    fn setup_logging(&mut self) -> Result<()> {
        let log_level: Level = self.config.global.log_level.parse()?;

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
            write_atomic(&key_path, self.keychain.to_bytes()?)?;
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

    fn setup_config(&mut self) -> Result<()> {
        let config_path = self.path.join("config.toml");
        let wallet_config_path = self.path.join("wallets.toml");
        let network_list_path = self.path.join("networks.toml");

        if config_path.try_exists()? {
            let config_text = fs::read_to_string(&config_path)?;

            if is_old_config(&config_text) {
                let old_config: OldConfig = parse_config_file(&config_path, &config_text)?;
                let (config, wallet_config) = migrate_config(old_config)?;
                self.config = config;
                write_atomic(&config_path, toml::to_string_pretty(&self.config)?)?;
                write_atomic(&wallet_config_path, toml::to_string_pretty(&wallet_config)?)?;
            } else {
                self.config = parse_config_file(&config_path, &config_text)?;
            }
        } else {
            write_atomic(&config_path, toml::to_string_pretty(&self.config)?)?;
        }

        // This is loaded separately, so that a missing main config doesn't overwrite the wallets.
        if wallet_config_path.try_exists()? {
            let wallet_config_text = fs::read_to_string(&wallet_config_path)?;
            self.wallet_config = parse_config_file(&wallet_config_path, &wallet_config_text)?;
        } else {
            write_atomic(
                &wallet_config_path,
                toml::to_string_pretty(&self.wallet_config)?,
            )?;
        }

        if network_list_path.try_exists()? {
            let text = fs::read_to_string(&network_list_path)?;

            // An empty file isn't an old network list, since migrating it would remove every network.
            if let Some(old_network_list) = toml::from_str::<IndexMap<String, OldNetwork>>(&text)
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
        } else {
            write_atomic(
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

    pub fn network(&self) -> &Network {
        if let Some(wallet) = self.wallet_config()
            && let Some(network) = &wallet.network
        {
            return self
                .network_list
                .by_name(network)
                .expect("network not found");
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

    fn ensure_writable(&self) -> Result<()> {
        if self.writable {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    pub fn save_config(&self) -> Result<()> {
        self.ensure_writable()?;

        let config = toml::to_string_pretty(&self.config)?;
        write_atomic(&self.path.join("config.toml"), config)?;
        let wallet_config = toml::to_string_pretty(&self.wallet_config)?;
        write_atomic(&self.path.join("wallets.toml"), wallet_config)?;
        let network_list = toml::to_string_pretty(&self.network_list)?;
        write_atomic(&self.path.join("networks.toml"), network_list)?;
        Ok(())
    }

    pub fn save_keychain(&self) -> Result<()> {
        self.ensure_writable()?;

        write_atomic(&self.path.join("keys.bin"), self.keychain.to_bytes()?)?;
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

fn parse_config_file<T: DeserializeOwned>(path: &Path, text: &str) -> Result<T> {
    toml::from_str(text).map_err(|error| Error::InvalidConfig {
        path: path.display().to_string(),
        error,
    })
}

/// Writes to a temporary file in the same directory, then renames it over the destination, so
/// that a crash or power loss can't leave the file truncated or empty.
fn write_atomic(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(dir)?;
    file.write_all(contents.as_ref())?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|error| error.error)?;

    // Sync the directory too, so that the rename itself survives a power loss. The file has
    // already been replaced at this point, so a filesystem that doesn't support this is fine.
    #[cfg(unix)]
    if let Err(error) = fs::File::open(dir).and_then(|dir| dir.sync_all()) {
        warn!("Failed to sync directory {}: {error}", dir.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn load(path: &Path) -> Result<Sage> {
        let mut sage = Sage::new(path, true);
        sage.writable = true;
        sage.setup_keys()?;
        sage.setup_config()?;
        sage.recover_missing_wallets()?;
        Ok(sage)
    }

    fn add_key(sage: &mut Sage, seed: u8) -> Result<u32> {
        let master_pk = SecretKey::from_seed(&[seed; 32]).public_key();
        Ok(sage.keychain.add_public_key(&master_pk)?)
    }

    fn save_named_wallet(path: &Path) -> Result<()> {
        let mut sage = load(path)?;
        sage.wallet_config.wallets.push(WalletEntry {
            name: "Named".to_string(),
            fingerprint: 1_000_000,
            ..Default::default()
        });
        sage.save_config()
    }

    #[test]
    fn test_empty_config_is_not_migrated() -> Result<()> {
        let dir = TempDir::new()?;
        save_named_wallet(dir.path())?;

        // An interrupted write can leave the config empty
        fs::write(dir.path().join("config.toml"), "")?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].name, "Named");

        Ok(())
    }

    #[test]
    fn test_missing_config_keeps_wallet_config() -> Result<()> {
        let dir = TempDir::new()?;
        save_named_wallet(dir.path())?;

        fs::remove_file(dir.path().join("config.toml"))?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].name, "Named");

        Ok(())
    }

    #[test]
    fn test_missing_wallet_config_is_created() -> Result<()> {
        let dir = TempDir::new()?;
        save_named_wallet(dir.path())?;

        fs::remove_file(dir.path().join("wallets.toml"))?;

        let sage = load(dir.path())?;
        assert!(sage.wallet_config.wallets.is_empty());
        assert!(dir.path().join("wallets.toml").try_exists()?);

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
    fn test_empty_network_list_is_not_migrated() -> Result<()> {
        let dir = TempDir::new()?;
        drop(load(dir.path())?);

        fs::write(dir.path().join("networks.toml"), "")?;

        let sage = load(dir.path())?;
        assert_eq!(sage.network().name, "mainnet");

        Ok(())
    }

    #[test]
    fn test_recover_wallets_missing_from_config() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let named = add_key(&mut sage, 1)?;
        let lost = add_key(&mut sage, 2)?;
        sage.save_keychain()?;
        drop(sage);

        fs::write(
            dir.path().join("wallets.toml"),
            format!("[[wallets]]\nname = \"Named\"\nfingerprint = {named}\n"),
        )?;

        let sage = load(dir.path())?;
        let wallets = &sage.wallet_config.wallets;
        assert_eq!(wallets.len(), 2);
        assert_eq!(wallets[0].name, "Named");
        assert_eq!(wallets[1].fingerprint, lost);

        // The recovered entry is saved
        let text = fs::read_to_string(dir.path().join("wallets.toml"))?;
        assert!(text.contains(&lost.to_string()));

        Ok(())
    }

    #[test]
    fn test_recover_wallets_when_wallet_config_deleted() -> Result<()> {
        let dir = TempDir::new()?;

        let mut sage = load(dir.path())?;
        let fingerprint = add_key(&mut sage, 1)?;
        sage.save_keychain()?;
        drop(sage);

        fs::remove_file(dir.path().join("wallets.toml"))?;

        let sage = load(dir.path())?;
        assert_eq!(sage.wallet_config.wallets.len(), 1);
        assert_eq!(sage.wallet_config.wallets[0].fingerprint, fingerprint);

        Ok(())
    }

    #[test]
    fn test_write_atomic_replaces_file() -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.path().join("file.toml");

        write_atomic(&path, "old")?;
        write_atomic(&path, "new")?;

        assert_eq!(fs::read_to_string(&path)?, "new");
        assert_eq!(fs::read_dir(dir.path())?.count(), 1);

        Ok(())
    }

    #[test]
    fn test_save_requires_initialization() -> Result<()> {
        let dir = TempDir::new()?;

        let sage = Sage::new(dir.path(), true);
        assert!(matches!(sage.save_config(), Err(Error::NotInitialized)));
        assert!(matches!(sage.save_keychain(), Err(Error::NotInitialized)));
        assert!(!dir.path().join("wallets.toml").try_exists()?);

        Ok(())
    }

    #[test]
    fn test_invalid_config_names_file() -> Result<()> {
        let dir = TempDir::new()?;
        drop(load(dir.path())?);

        fs::write(
            dir.path().join("wallets.toml"),
            "[[wallets]]\nname = Main\n",
        )?;

        let Err(Error::InvalidConfig { path, .. }) = load(dir.path()) else {
            panic!("expected an invalid config error");
        };
        assert!(path.ends_with("wallets.toml"));

        Ok(())
    }
}
