use std::{fs, str::FromStr};

use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{
    DeleteKey, DeleteKeyResponse, GenerateMnemonic, GenerateMnemonicResponse, GetKey,
    GetKeyResponse, GetKeys, GetKeysResponse, GetSecretKey, GetSecretKeyResponse, ImportKey,
    ImportKeyResponse, KeyInfo, KeyKind, Login, LoginResponse, Logout, LogoutResponse, RenameKey,
    RenameKeyResponse, Resync, ResyncResponse, SecretKeyInfo,
};

use crate::{Error, Result, Sage};

impl Sage {
    pub async fn login(&mut self, req: Login) -> Result<LoginResponse> {
        self.config.app.active_fingerprint = Some(req.fingerprint);
        self.save_config()?;
        self.switch_wallet().await?;
        Ok(LoginResponse {})
    }

    pub async fn logout(&mut self, _req: Logout) -> Result<LogoutResponse> {
        self.config.app.active_fingerprint = None;
        self.save_config()?;
        self.switch_wallet().await?;
        Ok(LogoutResponse {})
    }

    pub async fn resync(&mut self, req: Resync) -> Result<ResyncResponse> {
        let login = self.config.app.active_fingerprint == Some(req.fingerprint);

        if login {
            self.config.app.active_fingerprint = None;
            self.switch_wallet().await?;
        }

        self.delete_wallet_db(req.fingerprint)?;

        if login {
            self.config.app.active_fingerprint = Some(req.fingerprint);
            self.save_config()?;
            self.switch_wallet().await?;
        }

        Ok(ResyncResponse {})
    }

    pub fn generate_mnemonic(&self, req: GenerateMnemonic) -> Result<GenerateMnemonicResponse> {
        let mut rng = ChaCha20Rng::from_entropy();
        let mnemonic = if req.use_24_words {
            let entropy: [u8; 32] = rng.gen();
            Mnemonic::from_entropy(&entropy)?
        } else {
            let entropy: [u8; 16] = rng.gen();
            Mnemonic::from_entropy(&entropy)?
        };
        Ok(GenerateMnemonicResponse {
            mnemonic: mnemonic.to_string(),
        })
    }

    pub async fn import_key(&mut self, req: ImportKey) -> Result<ImportKeyResponse> {
        let mut key_hex = req.key.as_str();

        if key_hex.starts_with("0x") || key_hex.starts_with("0X") {
            key_hex = &key_hex[2..];
        }

        let fingerprint = if let Ok(bytes) = hex::decode(key_hex) {
            if let Ok(master_pk) = bytes.clone().try_into() {
                let master_pk = PublicKey::from_bytes(&master_pk)?;
                self.keychain.add_public_key(&master_pk)?
            } else if let Ok(master_sk) = bytes.try_into() {
                let master_sk = SecretKey::from_bytes(&master_sk)?;

                if req.save_secrets {
                    self.keychain.add_secret_key(&master_sk, b"")?
                } else {
                    self.keychain.add_public_key(&master_sk.public_key())?
                }
            } else {
                return Err(Error::InvalidKey);
            }
        } else {
            let mnemonic = Mnemonic::from_str(&req.key)?;

            if req.save_secrets {
                self.keychain.add_mnemonic(&mnemonic, b"")?
            } else {
                let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
                self.keychain.add_public_key(&master_sk.public_key())?
            }
        };

        let config = self.wallet_config_mut(fingerprint);
        config.name = req.name;
        self.config.app.active_fingerprint = Some(fingerprint);

        self.save_keychain()?;
        self.save_config()?;

        if req.login {
            self.switch_wallet().await?;
        }

        Ok(ImportKeyResponse { fingerprint })
    }

    pub fn delete_key(&mut self, req: DeleteKey) -> Result<DeleteKeyResponse> {
        self.keychain.remove(req.fingerprint);

        self.config
            .wallets
            .shift_remove(&req.fingerprint.to_string());
        if self.config.app.active_fingerprint == Some(req.fingerprint) {
            self.config.app.active_fingerprint = None;
        }

        self.save_keychain()?;
        self.save_config()?;

        let path = self.path.join("wallets").join(req.fingerprint.to_string());
        if path.try_exists()? {
            fs::remove_dir_all(path)?;
        }

        Ok(DeleteKeyResponse {})
    }

    pub fn rename_key(&mut self, req: RenameKey) -> Result<RenameKeyResponse> {
        let config = self.try_wallet_config_mut(req.fingerprint)?;
        config.name = req.name;
        self.save_config()?;

        Ok(RenameKeyResponse {})
    }

    pub fn get_key(&self, req: GetKey) -> Result<GetKeyResponse> {
        let fingerprint = req.fingerprint.or(self.config.app.active_fingerprint);

        let Some(fingerprint) = fingerprint else {
            return Ok(GetKeyResponse { key: None });
        };

        let name = self
            .config
            .wallets
            .get(&fingerprint.to_string())
            .map_or_else(|| "Unnamed".to_string(), |config| config.name.clone());

        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Ok(GetKeyResponse { key: None });
        };

        Ok(GetKeyResponse {
            key: Some(KeyInfo {
                name,
                fingerprint,
                public_key: hex::encode(master_pk.to_bytes()),
                kind: KeyKind::Bls,
                has_secrets: self.keychain.has_secret_key(fingerprint),
            }),
        })
    }

    pub fn get_secret_key(&self, req: GetSecretKey) -> Result<GetSecretKeyResponse> {
        let (mnemonic, Some(secret_key)) = self.keychain.extract_secrets(req.fingerprint, b"")?
        else {
            return Ok(GetSecretKeyResponse { secrets: None });
        };

        Ok(GetSecretKeyResponse {
            secrets: Some(SecretKeyInfo {
                mnemonic: mnemonic.map(|m| m.to_string()),
                secret_key: hex::encode(secret_key.to_bytes()),
            }),
        })
    }

    pub fn get_keys(&self, _req: GetKeys) -> Result<GetKeysResponse> {
        let mut keys = Vec::with_capacity(self.config.wallets.len());

        for (fingerprint, wallet) in &self.config.wallets {
            let fingerprint = fingerprint.parse::<u32>()?;

            let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
                continue;
            };

            keys.push(KeyInfo {
                name: wallet.name.clone(),
                fingerprint,
                public_key: hex::encode(master_pk.to_bytes()),
                kind: KeyKind::Bls,
                has_secrets: self.keychain.has_secret_key(fingerprint),
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

            keys.push(KeyInfo {
                name: "Unnamed".to_string(),
                fingerprint,
                public_key: hex::encode(master_pk.to_bytes()),
                kind: KeyKind::Bls,
                has_secrets: self.keychain.has_secret_key(fingerprint),
            });
        }

        Ok(GetKeysResponse { keys })
    }
}