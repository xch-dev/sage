use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chia::bls::master_to_wallet_unhardened_intermediate;
use chia_wallet_sdk::{create_rustls_connector, load_ssl_cert, Connector};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use sage_api::{SyncEvent as ApiEvent, Unit, TXCH, XCH};
use sage_config::{Config, Network, WalletConfig, MAINNET, TESTNET11};
use sage_database::Database;
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions, Timeouts, Wallet};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    ConnectOptions,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info, Level};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::{
    error::{Error, Result},
    models::{WalletInfo, WalletKind},
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    pub app_handle: AppHandle,
    pub path: PathBuf,
    pub config: Config,
    pub keychain: Keychain,
    pub networks: IndexMap<String, Network>,
    pub wallet: Option<Arc<Wallet>>,
    pub unit: Unit,
    pub initialized: bool,
    pub peer_state: Arc<Mutex<PeerState>>,
    pub command_sender: mpsc::Sender<SyncCommand>,
}

impl AppStateInner {
    pub fn new(app_handle: AppHandle, path: &Path) -> Self {
        Self {
            app_handle,
            path: path.to_path_buf(),
            config: Config::default(),
            keychain: Keychain::default(),
            networks: indexmap! {
                "mainnet".to_string() => MAINNET.clone(),
                "testnet11".to_string() => TESTNET11.clone(),
            },
            wallet: None,
            unit: XCH.clone(),
            initialized: false,
            peer_state: Arc::new(Mutex::new(PeerState::default())),
            command_sender: mpsc::channel(1).0,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.initialized = true;

        fs::create_dir_all(&self.path)?;

        self.setup_keys()?;
        self.setup_config()?;
        self.setup_networks()?;
        self.setup_logging()?;
        self.setup_sync_manager()?;
        self.switch_wallet().await?;

        info!("Initial setup complete");

        Ok(())
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

        if cfg!(mobile) {
            registry.try_init()?;
        } else {
            let stdout_layer = tracing_subscriber::fmt::layer().pretty();

            registry
                .with(stdout_layer.with_filter(filter!()))
                .try_init()?;
        }

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

    fn setup_sync_manager(&mut self) -> Result<()> {
        let connector = self.setup_ssl()?;

        let network_id = self.config.network.network_id.clone();
        let Some(network) = self.networks.get(network_id.as_str()).cloned() else {
            return Err(Error::unknown_network(&network_id));
        };

        let (sync_manager, command_sender, mut event_receiver) = SyncManager::new(
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

        let app_handle = self.app_handle.clone();
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                let event = match event {
                    SyncEvent::Start(ip) => ApiEvent::Start { ip: ip.to_string() },
                    SyncEvent::Stop => ApiEvent::Stop,
                    SyncEvent::Subscribed => ApiEvent::Subscribed,
                    SyncEvent::Derivation => ApiEvent::Derivation,
                    SyncEvent::CoinState => ApiEvent::CoinState,
                    SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
                    SyncEvent::CatInfo => ApiEvent::CatInfo,
                    SyncEvent::DidInfo => ApiEvent::DidInfo,
                    SyncEvent::NftData => ApiEvent::NftData,
                };
                if app_handle.emit("sync-event", event).is_err() {
                    break;
                }
            }

            Result::Ok(())
        });

        Ok(())
    }

    pub async fn switch_wallet(&mut self) -> Result<()> {
        let Some(fingerprint) = self.config.app.active_fingerprint else {
            self.wallet = None;

            let (sender, receiver) = oneshot::channel();

            self.command_sender
                .send(SyncCommand::SwitchWallet {
                    wallet: None,
                    callback: sender,
                })
                .await?;

            // receiver.await?;

            drop(receiver);

            return Ok(());
        };

        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Err(Error::unknown_fingerprint(fingerprint));
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
        if let Err(error) = sqlx::migrate!("../migrations").run(&pool).await {
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

            sqlx::migrate!("../migrations").run(&pool).await?;
        }

        let db = Database::new(pool);
        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            hex::decode(genesis_challenge)?.try_into()?,
        ));

        self.wallet = Some(wallet.clone());
        self.unit = match network_id.as_str() {
            "mainnet" => XCH.clone(),
            _ => TXCH.clone(),
        };

        let (sender, receiver) = oneshot::channel();

        self.command_sender
            .send(SyncCommand::SwitchWallet {
                wallet: Some(wallet),
                callback: sender,
            })
            .await?;

        // receiver.await?;

        drop(receiver);

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
            return Err(Error::not_logged_in());
        };

        if !self.keychain.contains(fingerprint) {
            return Err(Error::unknown_fingerprint(fingerprint));
        }

        let wallet = self.wallet.as_ref().ok_or(Error::not_logged_in())?;

        if wallet.fingerprint != fingerprint {
            return Err(Error::not_logged_in());
        }

        Ok(wallet.clone())
    }

    pub fn try_wallet_config(&self, fingerprint: u32) -> Result<&WalletConfig> {
        self.config
            .wallets
            .get(&fingerprint.to_string())
            .ok_or(Error::unknown_fingerprint(fingerprint))
    }

    pub fn try_wallet_config_mut(&mut self, fingerprint: u32) -> Result<&mut WalletConfig> {
        self.config
            .wallets
            .get_mut(&fingerprint.to_string())
            .ok_or(Error::unknown_fingerprint(fingerprint))
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

    pub fn wallets(&self) -> Result<Vec<WalletInfo>> {
        let mut wallets = Vec::with_capacity(self.config.wallets.len());

        for (fingerprint, wallet) in &self.config.wallets {
            let fingerprint = fingerprint.parse::<u32>()?;

            let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
                continue;
            };

            wallets.push(WalletInfo {
                name: wallet.name.clone(),
                fingerprint,
                public_key: hex::encode(master_pk.to_bytes()),
                kind: if self.keychain.has_secret_key(fingerprint) {
                    WalletKind::Hot
                } else {
                    WalletKind::Cold
                },
            });
        }

        for fingerprint in self
            .keychain
            .fingerprints()
            .filter(|fingerprint| !self.config.wallets.contains_key(&fingerprint.to_string()))
            .sorted()
        {
            let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
                continue;
            };

            wallets.push(WalletInfo {
                name: "Unnamed Wallet".to_string(),
                fingerprint,
                public_key: hex::encode(master_pk.to_bytes()),
                kind: if self.keychain.has_secret_key(fingerprint) {
                    WalletKind::Hot
                } else {
                    WalletKind::Cold
                },
            });
        }

        Ok(wallets)
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
