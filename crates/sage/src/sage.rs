use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{bls::master_to_wallet_unhardened_intermediate, protocol::Bytes32};
use chia_wallet_sdk::{create_rustls_connector, decode_address, load_ssl_cert, Connector};
use indexmap::{indexmap, IndexMap};
use sage_api::{Amount, Unit, XCH};
use sage_config::{Config, Network, WalletConfig, MAINNET, TESTNET11};
use sage_database::Database;
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts, Wallet};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
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
    pub keychain: Keychain,
    pub networks: IndexMap<String, Network>,
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
            keychain: Keychain::default(),
            networks: indexmap! {
                "mainnet".to_string() => MAINNET.clone(),
                "testnet11".to_string() => TESTNET11.clone(),
            },
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
        self.setup_networks()?;
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

        let log_level: Level = self.config.app.log_level.parse()?;

        // Create rotated log file
        let log_file = Builder::new()
            .filename_prefix("app.log")
            .rotation(Rotation::DAILY)
            .max_log_files(3)
            .build(log_dir)?;

        // Common filter string
        let filter_string = format!("{log_level},rustls=off,tungstenite=off");

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

        if config_path.try_exists()? {
            let text = fs::read_to_string(&config_path)?;
            self.config = toml::from_str(&text)?;
        } else {
            fs::write(&config_path, toml::to_string_pretty(&self.config)?)?;
        };

        Ok(())
    }

    fn setup_networks(&mut self) -> Result<()> {
        let networks_path = self.path.join("networks.toml");

        if networks_path.try_exists()? {
            let text = fs::read_to_string(&networks_path)?;
            self.networks = toml::from_str(&text)?;
        } else {
            fs::write(&networks_path, toml::to_string_pretty(&self.networks)?)?;
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

        let network_id = self.config.network.network_id.clone();
        let Some(network) = self.networks.get(network_id.as_str()).cloned() else {
            return Err(Error::UnknownNetwork);
        };

        let (sync_manager, command_sender, receiver) = SyncManager::new(
            SyncOptions {
                target_peers: if self.config.network.discover_peers {
                    self.config.network.target_peers as usize
                } else {
                    0
                },
                max_peer_age_seconds: 3600 * 8,
                dns_batch_size: 10,
                connection_batch_size: 30,
                timeouts: Timeouts::default(),
                testing: false,
            },
            self.peer_state.clone(),
            self.wallet.clone(),
            network_id,
            chia_wallet_sdk::Network {
                default_port: network.default_port,
                genesis_challenge: hex::decode(&network.genesis_challenge)?.try_into()?,
                dns_introducers: network.dns_introducers.clone(),
            },
            connector,
        );

        tokio::spawn(sync_manager.sync());
        self.command_sender = command_sender;

        Ok(receiver)
    }

    pub async fn switch_wallet(&mut self) -> Result<()> {
        let Some(fingerprint) = self.config.app.active_fingerprint else {
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

        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            hex::decode(&self.network().genesis_challenge)?.try_into()?,
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

        let peer_path = peer_dir.join(format!("{}.bin", self.config.network.network_id));

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
                    trusted: peers.trusted.contains(&ip),
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

        for peer in state.peers() {
            peers.connections.insert(peer.socket_addr().ip());
        }

        for (&ip, &ban) in state.banned_peers() {
            peers.banned.insert(ip, ban);
        }

        for &ip in state.trusted_peers() {
            peers.trusted.insert(ip);
        }

        let peer_path = peer_dir.join(format!("{}.bin", self.config.network.network_id));
        fs::write(&peer_path, peers.to_bytes()?)?;

        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn parse_address(&self, input: String) -> Result<Bytes32> {
        let (puzzle_hash, prefix) = decode_address(&input)?;

        if prefix != self.network().address_prefix {
            return Err(Error::AddressPrefix(prefix));
        }

        Ok(puzzle_hash.into())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn parse_amount(&self, input: Amount) -> Result<u64> {
        let Some(amount) = input.to_u64() else {
            return Err(Error::InvalidAmount(input.to_string()));
        };

        Ok(amount)
    }

    pub async fn connect_to_database(&self, fingerprint: u32) -> Result<SqlitePool> {
        let path = self.wallet_db_path(fingerprint)?;

        let pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path.display()))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace),
            )
            .await?;

        sqlx::migrate!("../../migrations").run(&pool).await?;

        Ok(pool)
    }

    fn wallet_db_path(&self, fingerprint: u32) -> Result<PathBuf> {
        let path = self.path.join("wallets").join(fingerprint.to_string());
        fs::create_dir_all(&path)?;
        let network_id = &self.config.network.network_id;
        let path = path.join(format!("{network_id}.sqlite"));
        Ok(path)
    }

    pub fn network(&self) -> &Network {
        self.networks
            .get(&self.config.network.network_id)
            .expect("network not found")
    }

    pub fn wallet(&self) -> Result<Arc<Wallet>> {
        let Some(fingerprint) = self.config.app.active_fingerprint else {
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

    pub fn try_wallet_config(&self, fingerprint: u32) -> Result<&WalletConfig> {
        self.config
            .wallets
            .get(&fingerprint.to_string())
            .ok_or(Error::UnknownFingerprint)
    }

    pub fn try_wallet_config_mut(&mut self, fingerprint: u32) -> Result<&mut WalletConfig> {
        self.config
            .wallets
            .get_mut(&fingerprint.to_string())
            .ok_or(Error::UnknownFingerprint)
    }

    pub fn wallet_config_mut(&mut self, fingerprint: u32) -> &mut WalletConfig {
        self.config
            .wallets
            .entry(fingerprint.to_string())
            .or_default()
    }

    pub fn save_config(&self) -> Result<()> {
        let config = toml::to_string_pretty(&self.config)?;
        fs::write(self.path.join("config.toml"), config)?;
        Ok(())
    }

    pub fn save_keychain(&self) -> Result<()> {
        fs::write(self.path.join("keys.bin"), self.keychain.to_bytes()?)?;
        Ok(())
    }
}
