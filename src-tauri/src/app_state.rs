use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use bip39::Mnemonic;
use chia::bls::{master_to_wallet_unhardened_intermediate, PublicKey, SecretKey};
use chia_wallet_sdk::{create_rustls_connector, load_ssl_cert, Network, NetworkId};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sage_api::{Unit, XCH};
use sage_config::{Config, WalletConfig};
use sage_database::Database;
use sage_keychain::{encrypt, KeyData, SecretKeyData};
use sage_wallet::{PeerState, SyncEvent, SyncManager, SyncOptions, Wallet};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    ConnectOptions,
};
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    error::{Error, Result},
    models::{SyncEvent as SyncEventData, WalletInfo, WalletKind},
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    pub app_handle: AppHandle,
    pub rng: ChaCha20Rng,
    pub path: PathBuf,
    pub config: Config,
    pub networks: IndexMap<String, Network>,
    pub keys: HashMap<u32, KeyData>,
    wallet: Option<Arc<Wallet>>,
    unit: Unit,
    pub sync_task: Option<JoinHandle<()>>,
    pub peer_state: Arc<Mutex<PeerState>>,
    pub initialized: bool,
}

impl AppStateInner {
    pub fn new(app_handle: AppHandle, path: &Path) -> Self {
        Self {
            app_handle,
            rng: ChaCha20Rng::from_entropy(),
            path: path.to_path_buf(),
            config: Config::default(),
            networks: indexmap! {
                "mainnet".to_string() => Network::default_mainnet(),
                "testnet11".to_string() => Network::default_testnet11(),
            },
            keys: HashMap::new(),
            wallet: None,
            unit: XCH.clone(),
            sync_task: None,
            peer_state: Arc::new(Mutex::new(PeerState::default())),
            initialized: false,
        }
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn reset_sync_task(&mut self, reset_peers: bool) -> Result<()> {
        if reset_peers {
            self.peer_state = Arc::new(Mutex::new(PeerState::default()));
        }

        if let Some(task) = self.sync_task.take() {
            task.abort();
        }

        let network_id = self.config.network.network_id.clone();
        let Some(network) = self.networks.get(network_id.as_str()).cloned() else {
            return Err(Error::unknown_network(&network_id));
        };

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

        let connector = create_rustls_connector(&cert)?;

        let (sync_manager, mut sync_receiver) = SyncManager::new(
            SyncOptions {
                target_peers: self.config.network.target_peers as usize,
                find_peers: self.config.network.discover_peers,
            },
            self.peer_state.clone(),
            self.wallet.clone(),
            NetworkId::Custom(network_id),
            network,
            connector,
        );

        self.sync_task = Some(tokio::spawn(sync_manager.sync()));

        // TODO: Should this task be aborted? It should get cleaned up automatically anyways I think.
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            while let Some(event) = sync_receiver.recv().await {
                let event = match event {
                    SyncEvent::Start(ip) => SyncEventData::Start { ip: ip.to_string() },
                    SyncEvent::Stop => SyncEventData::Stop,
                    SyncEvent::Subscribed => SyncEventData::Subscribed,
                    SyncEvent::CoinUpdate => SyncEventData::CoinUpdate,
                    SyncEvent::PuzzleUpdate => SyncEventData::PuzzleUpdate,
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
            self.reset_sync_task(false)?;
            return Ok(());
        };

        let key = self.keys.get(&fingerprint).cloned();

        let Some(key) = key else {
            return Err(Error::unknown_fingerprint(fingerprint));
        };

        let _wallet_config = self
            .config
            .wallets
            .get(&fingerprint.to_string())
            .cloned()
            .unwrap_or_default();

        let master_pk_bytes = match key {
            KeyData::Public { master_pk } | KeyData::Secret { master_pk, .. } => master_pk,
        };

        let master_pk = PublicKey::from_bytes(&master_pk_bytes)?;
        let intermediate_pk = master_to_wallet_unhardened_intermediate(&master_pk);

        let path = self.path.join("wallets").join(fingerprint.to_string());
        fs::create_dir_all(&path)?;

        let network_id = &self.config.network.network_id;
        let genesis_challenge = self.networks[network_id].genesis_challenge;
        let path = path.join(format!("{network_id}.sqlite"));
        let pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path.display()))?
                    .journal_mode(SqliteJournalMode::Wal)
                    .log_statements(log::LevelFilter::Trace),
            )
            .await?;
        sqlx::migrate!("../migrations").run(&pool).await?;

        let db = Database::new(pool);
        let wallet = Arc::new(Wallet::new(
            db.clone(),
            fingerprint,
            intermediate_pk,
            genesis_challenge,
        ));

        self.wallet = Some(wallet.clone());

        self.reset_sync_task(false)?;

        Ok(())
    }

    pub fn wallet(&self) -> Result<Arc<Wallet>> {
        let Some(fingerprint) = self.config.app.active_fingerprint else {
            return Err(Error::not_logged_in());
        };

        if !self.keys.contains_key(&fingerprint) {
            return Err(Error::unknown_fingerprint(fingerprint));
        }

        let wallet = self.wallet.as_ref().ok_or(Error::not_logged_in())?;

        if wallet.fingerprint != fingerprint {
            return Err(Error::not_logged_in());
        }

        Ok(wallet.clone())
    }

    pub fn prefix(&self) -> &'static str {
        match self.config.network.network_id.as_str() {
            "mainnet" => "xch",
            _ => "txch",
        }
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
        self.keys.remove(&fingerprint);

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
            let Some(key) = self.keys.get(&fingerprint) else {
                continue;
            };
            wallets.push(WalletInfo {
                name: wallet.name.clone(),
                fingerprint,
                public_key: hex::encode(match key {
                    KeyData::Public { master_pk } | KeyData::Secret { master_pk, .. } => master_pk,
                }),
                kind: match key {
                    KeyData::Public { .. } => WalletKind::Cold,
                    KeyData::Secret { .. } => WalletKind::Hot,
                },
            });
        }

        for fingerprint in self
            .keys
            .keys()
            .copied()
            .filter(|fingerprint| !self.config.wallets.contains_key(&fingerprint.to_string()))
            .sorted()
        {
            let key = self.keys.get(&fingerprint).expect("expected key data");
            wallets.push(WalletInfo {
                name: "Unnamed Wallet".to_string(),
                fingerprint,
                public_key: hex::encode(match key {
                    KeyData::Public { master_pk } | KeyData::Secret { master_pk, .. } => master_pk,
                }),
                kind: match key {
                    KeyData::Public { .. } => WalletKind::Cold,
                    KeyData::Secret { .. } => WalletKind::Hot,
                },
            });
        }

        Ok(wallets)
    }

    pub fn import_public_key(&mut self, master_pk: &PublicKey) -> Result<u32> {
        let fingerprint = master_pk.get_fingerprint();
        self.keys.insert(
            fingerprint,
            KeyData::Public {
                master_pk: master_pk.to_bytes(),
            },
        );
        self.save_keychain()?;
        Ok(fingerprint)
    }

    pub fn import_secret_key(&mut self, master_sk: &SecretKey) -> Result<u32> {
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();
        let encrypted = encrypt(
            b"",
            &mut self.rng,
            &SecretKeyData(master_sk.to_bytes().to_vec()),
        )?;
        self.keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: false,
                encrypted,
            },
        );
        self.save_keychain()?;
        Ok(fingerprint)
    }

    pub fn import_mnemonic(&mut self, mnemonic: &Mnemonic) -> Result<u32> {
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();
        let encrypted = encrypt(b"", &mut self.rng, &SecretKeyData(entropy))?;
        self.keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: true,
                encrypted,
            },
        );
        self.save_keychain()?;
        Ok(fingerprint)
    }

    pub fn save_config(&self) -> Result<()> {
        let config = toml::to_string_pretty(&self.config)?;
        fs::write(self.path.join("config.toml"), config)?;
        Ok(())
    }

    pub fn save_keychain(&self) -> Result<()> {
        let data = bincode::serialize(&self.keys)?;
        fs::write(self.path.join("keys.bin"), data)?;
        Ok(())
    }
}
