use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use itertools::Itertools;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sage::{encrypt, KeyData, SecretKeyData};
use std::{collections::HashMap, fs, path::PathBuf};
use tokio::sync::Mutex;

use crate::{
    config::Config,
    error::Result,
    models::{WalletInfo, WalletKind},
};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    rng: ChaCha20Rng,
    key_path: PathBuf,
    config_path: PathBuf,
}

impl AppStateInner {
    pub fn new(path: PathBuf) -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
            key_path: path.join("keys.bin"),
            config_path: path.join("config.toml"),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        if !self.key_path.try_exists()? {
            let keys = HashMap::<u32, KeyData>::new();
            fs::write(&self.key_path, bincode::serialize(&keys)?)?;
        }
        if !self.config_path.try_exists()? {
            let config = Config::default();
            fs::write(&self.config_path, toml::to_string_pretty(&config)?)?;
        }
        Ok(())
    }

    pub fn login_wallet(&self, fingerprint: u32) -> Result<()> {
        let mut config = self.load_config()?;
        config.active_wallet = Some(fingerprint);
        self.save_config(config)?;
        Ok(())
    }

    pub fn logout_wallet(&self) -> Result<()> {
        let mut config = self.load_config()?;
        config.active_wallet = None;
        self.save_config(config)?;
        Ok(())
    }

    pub fn active_wallet(&self) -> Result<Option<WalletInfo>> {
        let config = self.load_config()?;
        let keychain = self.load_keychain()?;

        let fingerprint = match config.active_wallet {
            Some(fingerprint) => fingerprint,
            None => return Ok(None),
        };

        let name = config
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
        let config = self.load_config()?;

        let mut wallets = Vec::with_capacity(config.wallets.len());

        for (fingerprint, wallet) in &config.wallets {
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
            .filter(|fingerprint| !config.wallets.contains_key(&fingerprint.to_string()))
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

    pub fn rename_wallet(&self, fingerprint: u32, name: String) -> Result<()> {
        let mut config = self.load_config()?;
        let wallet_config = config.wallets.entry(fingerprint.to_string()).or_default();
        wallet_config.name = name;
        self.save_config(config)?;
        Ok(())
    }

    pub fn delete_wallet(&self, fingerprint: u32) -> Result<()> {
        let mut keys = self.load_keychain()?;
        let mut config = self.load_config()?;
        keys.remove(&fingerprint);
        config.wallets.shift_remove(&fingerprint.to_string());
        if config.active_wallet == Some(fingerprint) {
            config.active_wallet = None;
        }
        self.save_keychain(keys)?;
        self.save_config(config)?;
        Ok(())
    }

    fn load_config(&self) -> Result<Config> {
        let config = fs::read_to_string(&self.config_path)?;
        Ok(toml::from_str(&config)?)
    }

    fn save_config(&self, config: Config) -> Result<()> {
        let config = toml::to_string_pretty(&config)?;
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
