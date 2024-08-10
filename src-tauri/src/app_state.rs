use bip39::Mnemonic;
use chia::bls::{derive_keys::master_to_wallet_unhardened_intermediate, PublicKey, SecretKey};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sage::{encrypt, Database, KeyData, SecretKeyData};
use sage_client::{Network, Peer};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::{
    config::{Config, WalletConfig},
    error::{Error, Result},
    models::{WalletInfo, WalletKind},
    wallet::Wallet,
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    app_handle: AppHandle,
    rng: ChaCha20Rng,
    path: PathBuf,
    config: Config,
    networks: IndexMap<String, Network>,
    keys: HashMap<u32, KeyData>,
    wallet: Option<Wallet>,
    peers: HashMap<SocketAddr, Peer>,
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
            peers: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        fs::create_dir_all(&self.path)?;

        let key_path = self.path.join("keys.bin");
        let config_path = self.path.join("config.toml");
        let networks_path = self.path.join("networks.toml");

        if !key_path.try_exists()? {
            fs::write(&key_path, bincode::serialize(&self.keys)?)?;
        } else {
            let data = fs::read(&key_path)?;
            self.keys = bincode::deserialize(&data)?;
        }

        if !config_path.try_exists()? {
            fs::write(&config_path, toml::to_string_pretty(&self.config)?)?;
        } else {
            let text = fs::read_to_string(&config_path)?;
            self.config = toml::from_str(&text)?;
        };

        if !networks_path.try_exists()? {
            fs::write(&networks_path, toml::to_string_pretty(&self.networks)?)?;
        } else {
            let text = fs::read_to_string(&networks_path)?;
            self.networks = toml::from_str(&text)?;
        }

        if let Some(fingerprint) = self.config.wallet.active_fingerprint {
            self.login_wallet(fingerprint).await?;
        }

        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
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

    pub fn networks(&self) -> &IndexMap<String, Network> {
        &self.networks
    }

    pub fn keys(&self) -> &HashMap<u32, KeyData> {
        &self.keys
    }

    pub fn wallet(&self) -> Option<&Wallet> {
        self.wallet.as_ref()
    }

    pub async fn login_wallet(&mut self, fingerprint: u32) -> Result<()> {
        self.config.wallet.active_fingerprint = Some(fingerprint);
        self.save_config()?;

        let key = self.keys.get(&fingerprint).cloned();

        let Some(key) = key else {
            return Err(Error::Fingerprint(fingerprint));
        };

        let wallet_config = self
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
        let wallet = Wallet::new(fingerprint, intermediate_pk, db);

        wallet
            .initial_sync(wallet_config.derivation_batch_size)
            .await?;

        self.wallet = Some(wallet);

        Ok(())
    }

    pub fn logout_wallet(&mut self) -> Result<()> {
        self.config.wallet.active_fingerprint = None;
        self.save_config()?;
        Ok(())
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
