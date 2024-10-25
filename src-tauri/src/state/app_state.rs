use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use chia_wallet_sdk::{create_rustls_connector, load_ssl_cert, Connector};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use sage_api::SyncEvent as ApiEvent;
use sage_config::{Config, Network, WalletConfig, MAINNET, TESTNET11};
use sage_keychain::Keychain;
use sage_wallet::{PeerState, SyncCommand, SyncEvent, SyncManager, SyncOptions};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::{
    error::{Error, Result},
    models::{WalletInfo, WalletKind},
};

use super::WalletState;

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    pub app_handle: AppHandle,
    pub path: PathBuf,
    pub config: Config,
    pub keychain: Keychain,
    pub networks: IndexMap<String, Network>,
    pub initialized: bool,
    pub peer_state: Arc<Mutex<PeerState>>,
    pub command_sender: mpsc::Sender<SyncCommand>,
    pub wallet: Option<WalletState>,
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

        let log_level = self.config.app.log_level.parse()?;

        let log_file = Builder::new()
            .filename_prefix("app.log")
            .rotation(Rotation::DAILY)
            .max_log_files(3)
            .build(log_dir.as_path())?;

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(log_file)
            .with_ansi(false)
            .with_target(false)
            .compact();

        // TODO: Fix ANSI better
        #[cfg(not(mobile))]
        let stdout_layer = tracing_subscriber::fmt::layer().pretty();

        let registry = tracing_subscriber::registry()
            .with(file_layer.with_filter(LevelFilter::from_level(log_level)));

        #[cfg(not(mobile))]
        {
            registry
                .with(stdout_layer.with_filter(LevelFilter::from_level(log_level)))
                .init();
        }

        #[cfg(mobile)]
        registry.init();

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
                target_peers: self.config.network.target_peers as usize,
                discover_peers: self.config.network.discover_peers,
                max_peer_age_seconds: 3600 * 8,
                max_peers_for_dns: 0,
                dns_batch_size: 10,
                connection_batch_size: 30,
                sync_delay: Duration::from_secs(1),
                connection_timeout: Duration::from_secs(3),
                initial_peak_timeout: Duration::from_secs(2),
                remove_subscription_timeout: Duration::from_secs(3),
                request_peers_timeout: Duration::from_secs(3),
                dns_timeout: Duration::from_secs(3),
            },
            self.peer_state.clone(),
            self.wallet.as_ref().map(|state| state.inner.clone()),
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

            self.command_sender
                .send(SyncCommand::SwitchWallet { wallet: None })
                .await?;

            return Ok(());
        };

        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Err(Error::unknown_fingerprint(fingerprint));
        };

        let wallet_path = self.path.join("wallets").join(fingerprint.to_string());
        let network_id = self.config.network.network_id.clone();
        let genesis_challenge =
            hex::decode(&self.networks[&network_id].genesis_challenge)?.try_into()?;

        let wallet = WalletState::open(
            self.app_handle.clone(),
            wallet_path,
            network_id,
            genesis_challenge,
            master_pk,
        )
        .await?;
        let inner = wallet.inner.clone();
        self.wallet = Some(wallet);

        self.command_sender
            .send(SyncCommand::SwitchWallet {
                wallet: Some(inner),
            })
            .await?;

        Ok(())
    }

    pub fn network(&self) -> &Network {
        self.networks
            .get(&self.config.network.network_id)
            .expect("network not found")
    }

    pub fn wallet(&self) -> Result<&WalletState> {
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

        Ok(wallet)
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
