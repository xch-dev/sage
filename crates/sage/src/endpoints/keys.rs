use std::{fs, str::FromStr};

use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use sage_api::{
    DeleteKey, DeleteKeyResponse, ImportKey, ImportKeyResponse, Login, LoginResponse, Logout,
    LogoutResponse, Resync, ResyncResponse,
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
}

// #[command]
// #[specta]
// pub async fn wallet_list(state: State<'_, AppState>) -> Result<Vec<WalletInfo>> {
//     let state = state.lock().await;
//     state.wallets()
// }

// #[command]
// #[specta]
// pub async fn active_wallet(state: State<'_, AppState>) -> Result<Option<WalletInfo>> {
//     let state = state.lock().await;

//     let Some(fingerprint) = state.config.app.active_fingerprint else {
//         return Ok(None);
//     };

//     let name = state
//         .config
//         .wallets
//         .get(&fingerprint.to_string())
//         .map_or_else(
//             || "Unnamed Wallet".to_string(),
//             |config| config.name.clone(),
//         );

//     let Some(master_pk) = state.keychain.extract_public_key(fingerprint)? else {
//         return Ok(None);
//     };

//     Ok(Some(WalletInfo {
//         name,
//         fingerprint,
//         public_key: hex::encode(master_pk.to_bytes()),
//         kind: if state.keychain.has_secret_key(fingerprint) {
//             WalletKind::Hot
//         } else {
//             WalletKind::Cold
//         },
//     }))
// }

// #[command]
// #[specta]
// pub async fn get_wallet_secrets(
//     state: State<'_, AppState>,
//     fingerprint: u32,
// ) -> Result<Option<WalletSecrets>> {
//     let state = state.lock().await;

//     let (mnemonic, Some(secret_key)) = state.keychain.extract_secrets(fingerprint, b"")? else {
//         return Ok(None);
//     };

//     Ok(Some(WalletSecrets {
//         mnemonic: mnemonic.map(|m| m.to_string()),
//         secret_key: hex::encode(secret_key.to_bytes()),
//     }))
// }
