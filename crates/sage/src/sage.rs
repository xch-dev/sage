use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chia::bls::master_to_wallet_unhardened_intermediate;
use chia_wallet_sdk::{create_rustls_connector, load_ssl_cert, Connector};
use indexmap::{indexmap, IndexMap};
use sage_config::{Config, Network, WalletConfig, MAINNET, TESTNET11};
use sage_database::Database;
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts, Wallet};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    ConnectOptions,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, Level};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::{Error, Result};

#[derive(Debug)]
pub struct Sage {
    pub path: PathBuf,
    pub config: Config,
    pub keychain: Keychain,
    pub networks: IndexMap<String, Network>,
    pub wallet: Option<Arc<Wallet>>,
    pub peer_state: Arc<Mutex<PeerState>>,
    pub command_sender: mpsc::Sender<SyncCommand>,
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

        info!("Sage wallet initialized");

        Ok(receiver)
    }

    fn setup_logging(&mut self) -> Result<()> {
        let log_dir = self.path.join("log");

        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }

        let log_level: Level = self.config.app.log_level.parse()?;

        let log_file = Builder::new()
            .filename_prefix("app.log")
            .rotation(Rotation::DAILY)
            .max_log_files(3)
            .build(log_dir.as_path())?;

        macro_rules! filter {
            () => {
                EnvFilter::new(format!(
                    "{},rustls=off,tungstenite=off",
                    log_level.to_string()
                ))
            };
        }

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(log_file)
            .with_ansi(false)
            .with_target(false)
            .compact()
            .with_filter(filter!());

        let registry = tracing_subscriber::registry().with(file_layer);

        let stdout_layer = tracing_subscriber::fmt::layer().pretty();

        registry
            .with(stdout_layer.with_filter(filter!()))
            .try_init()?;

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

        let path = self.wallet_db_path(fingerprint)?;
        let network_id = &self.config.network.network_id;
        let genesis_challenge = &self.networks[network_id].genesis_challenge;

        let mut pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path.display()))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace),
            )
            .await?;

        // TODO: Remove this before out of beta.
        if let Err(error) = sqlx::migrate!("../../migrations").run(&pool).await {
            error!("Error migrating database, dropping database: {error:?}");

            pool.close().await;
            drop(pool);

            fs::remove_file(&path)?;

            pool = SqlitePoolOptions::new()
                .connect_with(
                    SqliteConnectOptions::from_str(&format!(
                        "sqlite://{}?mode=rwc",
                        path.display()
                    ))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace),
                )
                .await?;

            sqlx::migrate!("../../migrations").run(&pool).await?;
        }

        let db = Database::new(pool);
        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            hex::decode(genesis_challenge)?.try_into()?,
        ));

        self.wallet = Some(wallet.clone());

        self.command_sender
            .send(SyncCommand::SwitchWallet {
                wallet: Some(wallet),
            })
            .await?;

        Ok(())
    }

    pub fn delete_wallet_db(&self, fingerprint: u32) -> Result<()> {
        let path = self.wallet_db_path(fingerprint)?;
        Ok(fs::remove_file(path)?)
    }

    pub fn wallet_db_path(&self, fingerprint: u32) -> Result<PathBuf> {
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

    pub fn delete_wallet(&mut self, fingerprint: u32) -> Result<()> {
        self.keychain.remove(fingerprint);

        self.config.wallets.shift_remove(&fingerprint.to_string());
        if self.config.app.active_fingerprint == Some(fingerprint) {
            self.config.app.active_fingerprint = None;
        }

        self.save_keychain()?;
        self.save_config()?;

        let path = self.path.join("wallets").join(fingerprint.to_string());
        if path.try_exists()? {
            fs::remove_dir_all(path)?;
        }

        Ok(())
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
