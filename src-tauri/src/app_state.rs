use bip39::Mnemonic;
use chia::bls::{derive_keys::master_to_wallet_unhardened_intermediate, PublicKey, SecretKey};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sage::{encrypt, Database, KeyData, Network, SecretKeyData};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

use crate::{
    config::{Config, WalletConfig},
    error::{Error, Result},
    models::{WalletInfo, WalletKind},
    wallet::Wallet,
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    rng: ChaCha20Rng,
    path: PathBuf,
    key_path: PathBuf,
    config_path: PathBuf,
    networks_path: PathBuf,
    db_path: PathBuf,
    config: Config,
    networks: IndexMap<String, Network>,
    wallet: Option<Wallet>,
}

impl AppStateInner {
    pub fn new(path: &Path) -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
            path: path.to_path_buf(),
            key_path: path.join("keys.bin"),
            config_path: path.join("config.toml"),
            networks_path: path.join("networks.toml"),
            config: Config::default(),
            networks: indexmap! {
                "mainnet".to_string() => Network::default_mainnet(),
                "testnet11".to_string() => Network::default_testnet11(),
            },
            db_path: path.join("wallets"),
            wallet: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if !self.path.try_exists()? {
            fs::create_dir_all(&self.path)?;
        }

        if !self.key_path.try_exists()? {
            let keys = HashMap::<u32, KeyData>::new();
            fs::write(&self.key_path, bincode::serialize(&keys)?)?;
        }

        if !self.config_path.try_exists()? {
            fs::write(&self.config_path, toml::to_string_pretty(&self.config)?)?;
        } else {
            let text = fs::read_to_string(&self.config_path)?;
            self.config = toml::from_str(&text)?;
        };

        if !self.networks_path.try_exists()? {
            fs::write(&self.networks_path, toml::to_string_pretty(&self.networks)?)?;
        } else {
            let text = fs::read_to_string(&self.networks_path)?;
            self.networks = toml::from_str(&text)?;
        }

        if !self.db_path.try_exists()? {
            fs::create_dir(&self.db_path)?;
        }

        if let Some(fingerprint) = self.config.active_wallet {
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

    pub fn wallet(&self) -> Option<&Wallet> {
        self.wallet.as_ref()
    }

    pub async fn login_wallet(&mut self, fingerprint: u32) -> Result<()> {
        self.config.active_wallet = Some(fingerprint);
        self.save_config()?;

        let keychain = self.load_keychain()?;
        let key = keychain.get(&fingerprint).cloned();

        if let Some(key) = key {
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

            let path = self.db_path.join(format!("{fingerprint}.sqlite"));
            let pool =
                SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path.display())).await?;
            sqlx::migrate!("../migrations").run(&pool).await?;

            let db = Database::new(pool);
            let wallet = Wallet::new(fingerprint, intermediate_pk, db);

            wallet
                .initial_sync(wallet_config.derivation_batch_size)
                .await?;

            self.wallet = Some(wallet);
        }

        Ok(())
    }

    pub fn logout_wallet(&mut self) -> Result<()> {
        self.config.active_wallet = None;
        self.save_config()?;
        Ok(())
    }

    pub fn active_wallet(&self) -> Result<Option<WalletInfo>> {
        let keychain = self.load_keychain()?;

        let fingerprint = match self.config.active_wallet {
            Some(fingerprint) => fingerprint,
            None => return Ok(None),
        };

        let name = self
            .config
            .wallets
            .get(&fingerprint.to_string())
            .map(|config| config.name.clone())
            .unwrap_or_else(|| "Unnamed Wallet".to_string());

        let Some(key) = keychain.get(&fingerprint) else {
            return Ok(None);
        };

        let kind = match key {
            KeyData::Public { .. } => WalletKind::Cold,
            KeyData::Secret { .. } => WalletKind::Hot,
        };

        Ok(Some(WalletInfo {
            name,
            fingerprint,
            kind,
        }))
    }

    pub fn wallets(&self) -> Result<Vec<WalletInfo>> {
        let keychain = self.load_keychain()?;

        let mut wallets = Vec::with_capacity(self.config.wallets.len());

        for (fingerprint, wallet) in &self.config.wallets {
            let fingerprint = fingerprint.parse::<u32>()?;
            let Some(key) = keychain.get(&fingerprint) else {
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

        for fingerprint in keychain
            .keys()
            .copied()
            .filter(|fingerprint| !self.config.wallets.contains_key(&fingerprint.to_string()))
            .sorted()
        {
            let key = keychain.get(&fingerprint).unwrap();
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
        let mut keys = self.load_keychain()?;
        let fingerprint = master_pk.get_fingerprint();
        keys.insert(
            fingerprint,
            KeyData::Public {
                master_pk: master_pk.to_bytes(),
            },
        );
        self.save_keychain(keys)?;
        Ok(fingerprint)
    }

    pub fn import_secret_key(&mut self, master_sk: &SecretKey) -> Result<u32> {
        let mut keys = self.load_keychain()?;
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();
        let encrypted = encrypt(
            b"",
            &mut self.rng,
            &SecretKeyData(master_sk.to_bytes().to_vec()),
        )?;
        keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: false,
                encrypted,
            },
        );
        self.save_keychain(keys)?;
        Ok(fingerprint)
    }

    pub fn import_mnemonic(&mut self, mnemonic: &Mnemonic) -> Result<u32> {
        let mut keys = self.load_keychain()?;
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();
        let encrypted = encrypt(b"", &mut self.rng, &SecretKeyData(entropy))?;
        keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: true,
                encrypted,
            },
        );
        self.save_keychain(keys)?;
        Ok(fingerprint)
    }

    pub fn delete_wallet(&mut self, fingerprint: u32) -> Result<()> {
        let mut keys = self.load_keychain()?;
        keys.remove(&fingerprint);
        self.config.wallets.shift_remove(&fingerprint.to_string());
        if self.config.active_wallet == Some(fingerprint) {
            self.config.active_wallet = None;
        }
        self.save_keychain(keys)?;
        self.save_config()?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let config = toml::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, config)?;
        Ok(())
    }

    fn load_keychain(&self) -> Result<HashMap<u32, KeyData>> {
        let data = fs::read(&self.key_path)?;
        Ok(bincode::deserialize(&data)?)
    }

    fn save_keychain(&self, keychain: HashMap<u32, KeyData>) -> Result<()> {
        let data = bincode::serialize(&keychain)?;
        fs::write(&self.key_path, data)?;
        Ok(())
    }
}
