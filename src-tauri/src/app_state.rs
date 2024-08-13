use bip39::Mnemonic;
use chia::{
    bls::{
        derive_keys::master_to_wallet_unhardened_intermediate, DerivableKey, PublicKey, SecretKey,
    },
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{create_tls_connector, load_ssl_cert};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sage::{encrypt, Database, KeyData, SecretKeyData};
use sage_client::Network;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    config::{Config, PeerMode, WalletConfig},
    error::{Error, Result},
    models::{WalletInfo, WalletKind},
    peer_discovery::{peer_discovery, PeerContext},
    sync_manager::SyncManager,
    wallet::Wallet,
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    pub app_handle: AppHandle,
    pub rng: ChaCha20Rng,
    pub path: PathBuf,
    pub config: Config,
    pub networks: IndexMap<String, Network>,
    pub keys: HashMap<u32, KeyData>,
    pub wallet: Option<Arc<Wallet>>,
    pub sync_manager: Arc<Mutex<SyncManager>>,
    peer_discovery_task: Option<JoinHandle<()>>,
}

impl AppStateInner {
    pub fn new(app_handle: AppHandle, path: &Path) -> Self {
        Self {
            app_handle: app_handle.clone(),
            rng: ChaCha20Rng::from_entropy(),
            path: path.to_path_buf(),
            config: Config::default(),
            networks: indexmap! {
                "mainnet".to_string() => Network::default_mainnet(),
                "testnet11".to_string() => Network::default_testnet11(),
            },
            keys: HashMap::new(),
            wallet: None,
            sync_manager: Arc::new(Mutex::new(SyncManager::new(app_handle))),
            peer_discovery_task: None,
        }
    }

    pub async fn setup_networking(&mut self, reset_peers: bool) -> Result<()> {
        if reset_peers {
            self.sync_manager = Arc::new(Mutex::new(SyncManager::new(self.app_handle.clone())));
            self.app_handle.emit("peer-update", ())?;
        }

        if let Some(task) = self.peer_discovery_task.take() {
            task.abort();
        }

        let network_id = self.config.network.network_id.clone();
        let Some(network) = self.networks.get(network_id.as_str()).cloned() else {
            return Err(Error::UnknownNetwork(network_id.clone()));
        };

        let ssl_dir = self.path.join("ssl");
        if !ssl_dir.try_exists()? {
            fs::create_dir_all(&ssl_dir)?;
        }

        let cert = load_ssl_cert(
            ssl_dir.join("wallet.crt").to_str().unwrap(),
            ssl_dir.join("wallet.key").to_str().unwrap(),
        )?;

        let tls_connector = create_tls_connector(&cert)?;

        if self.config.network.peer_mode == PeerMode::Automatic {
            self.peer_discovery_task = Some(tokio::spawn(peer_discovery(PeerContext {
                tls_connector,
                config: self.config.network.clone(),
                network,
                sync_manager: self.sync_manager.clone(),
                app_handle: self.app_handle.clone(),
            })));
        } else {
            self.peer_discovery_task = None;
        }

        self.setup_wallet().await?;

        Ok(())
    }

    pub async fn setup_wallet(&mut self) -> Result<()> {
        let Some(fingerprint) = self.config.wallet.active_fingerprint else {
            self.wallet = None;
            return Ok(());
        };

        let key = self.keys.get(&fingerprint).cloned();

        let Some(key) = key else {
            return Err(Error::Fingerprint(fingerprint));
        };

        let _wallet_config = self
            .config
            .wallets
            .get(&fingerprint.to_string())
            .cloned()
            .unwrap_or_default();

        let master_pk_bytes = match key {
            KeyData::Public { master_pk } => master_pk,
            KeyData::Secret { master_pk, .. } => master_pk,
        };

        let master_pk = PublicKey::from_bytes(&master_pk_bytes)?;
        let intermediate_pk = master_to_wallet_unhardened_intermediate(&master_pk);

        let path = self.path.join("wallets").join(fingerprint.to_string());
        fs::create_dir_all(&path)?;

        let network_id = &self.config.network.network_id;
        let path = path.join(format!("{network_id}.sqlite"));
        let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path.display())).await?;
        sqlx::migrate!("../migrations").run(&pool).await?;

        let db = Database::new(pool);
        let wallet = Wallet::new(
            fingerprint,
            intermediate_pk,
            db.clone(),
            self.networks[network_id].genesis_challenge.into(),
        );

        let mut tx = db.tx().await?;

        let index = tx.derivation_index(false).await?;

        for i in index..1000 {
            let pk = intermediate_pk.derive_unhardened(i).derive_synthetic();
            let p2_puzzle_hash = StandardArgs::curry_tree_hash(pk);
            tx.insert_derivation(p2_puzzle_hash.into(), i, false, pk)
                .await?;
        }

        tx.commit().await?;

        self.wallet = Some(Arc::new(wallet));
        self.sync_manager
            .lock()
            .await
            .switch_wallet(self.wallet.clone());

        Ok(())
    }

    pub fn try_wallet_config(&self, fingerprint: u32) -> Result<&WalletConfig> {
        self.config
            .wallets
            .get(&fingerprint.to_string())
            .ok_or(Error::Fingerprint(fingerprint))
    }

    pub fn try_wallet_config_mut(&mut self, fingerprint: u32) -> Result<&mut WalletConfig> {
        self.config
            .wallets
            .get_mut(&fingerprint.to_string())
            .ok_or(Error::Fingerprint(fingerprint))
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
        if self.config.wallet.active_fingerprint == Some(fingerprint) {
            self.config.wallet.active_fingerprint = None;
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
            let key = self.keys.get(&fingerprint).unwrap();
            wallets.push(WalletInfo {
                name: "Unnamed Wallet".to_string(),
                fingerprint,
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
