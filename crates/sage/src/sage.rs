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

use crate::{peers::Peers, Error, Result};

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
        }
    }

    pub async fn initialize(&mut self) -> Result<mpsc::Receiver<SyncEvent>> {
        fs::create_dir_all(&self.path)?;

        self.setup_keys()?;
        self.setup_config()?;
        self.setup_logging()?;

        let receiver = self.setup_sync_manager()?;
        self.switch_wallet().await?;
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
        };

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

        Ok(receiver)
    }

    pub async fn switch_network(&mut self) -> Result<()> {
        self.command_sender
            .send(SyncCommand::SwitchNetwork(self.network().clone()))
            .await?;

        Ok(())
    }

    pub async fn switch_wallet(&mut self) -> Result<()> {
        self.switch_network().await?;

        let Some(fingerprint) = self.config.global.fingerprint else {
            self.wallet = None;

            self.command_sender
                .send(SyncCommand::SwitchWallet { wallet: None })
                .await?;

            return Ok(());
        };

        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Err(Error::UnknownFingerprint);
        };

        let intermediate_pk = master_to_wallet_unhardened_intermediate(&master_pk);

        let pool = self.connect_to_database(fingerprint).await?;
        let db = Database::new(pool);
        db.run_rust_migrations().await?;

        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            self.network().genesis_challenge,
        ));

        self.wallet = Some(wallet.clone());
        self.unit = Unit {
            ticker: self.network().ticker.clone(),
            decimals: self.network().precision,
        };

        self.command_sender
            .send(SyncCommand::SwitchWallet {
                wallet: Some(wallet),
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

        sqlx::migrate!("../../migrations").run(&pool).await?;

        Ok(pool)
    }

    fn wallet_db_path(&self, fingerprint: u32) -> Result<PathBuf> {
        let path = self.path.join("wallets").join(fingerprint.to_string());
        fs::create_dir_all(&path)?;
        let path = path.join(format!("{}.sqlite", self.network_id()));
        Ok(path)
    }

    pub fn network(&self) -> &Network {
        if let Some(fingerprint) = self.config.global.fingerprint {
            if let Some(wallet) = self
                .wallet_config
                .wallets
                .iter()
                .find(|w| w.fingerprint == fingerprint)
            {
                if let Some(network) = &wallet.network {
                    return self
                        .network_list
                        .by_name(network)
                        .expect("network not found");
                }
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
}
