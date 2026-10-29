use std::{fs, str::FromStr};

use bip39::Mnemonic;
use chia_wallet_sdk::{
    chia::{
        bls::{
            DerivableKey, master_to_wallet_hardened_intermediate,
            master_to_wallet_unhardened_intermediate,
        },
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    prelude::*,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{
    AddKey, AddKeyResponse, DeleteDatabase, DeleteDatabaseResponse, DeleteKey, DeleteKeyResponse,
    DeleteWallet, DeleteWalletResponse, GenerateMnemonic, GenerateMnemonicResponse, GetKey,
    GetKeyResponse, GetKeys, GetKeysResponse, GetSecretKey, GetSecretKeyResponse, GetWallet,
    GetWalletAddress, GetWalletAddressResponse, GetWalletResponse, GetWallets, GetWalletsResponse,
    ImportWallet, ImportWalletResponse, KeyInfo, KeyKind, Login, LoginResponse, Logout,
    LogoutResponse, RenameWallet, RenameWalletResponse, Resync, ResyncResponse, SecretKeyInfo,
    SetWalletEmoji, SetWalletEmojiResponse, WalletInfo, WalletKind as ApiWalletKind,
};
use sage_config::{Wallet, WalletKind as ConfigWalletKind};
use sage_database::{Database, Derivation};
use sage_keychain::KeychainError;
use sage_wallet::WalletKind;
use sqlx::query;

use crate::{Error, Result, Sage};

impl Sage {
    pub async fn login(&mut self, req: Login) -> Result<LoginResponse> {
        // The keychain can hold keys that aren't wallets, and a wallet's key can be missing, so
        // the wallet is checked before it's saved, since a saved fingerprint is opened on every
        // start.
        self.wallet_kind(req.fingerprint)?;

        self.config.global.fingerprint = Some(req.fingerprint);
        self.save_config()?;
        self.switch_wallet().await?;
        Ok(LoginResponse {})
    }

    pub async fn logout(&mut self, _req: Logout) -> Result<LogoutResponse> {
        self.config.global.fingerprint = None;
        self.save_config()?;
        self.switch_wallet().await?;
        Ok(LogoutResponse {})
    }

    pub async fn resync(&mut self, req: Resync) -> Result<ResyncResponse> {
        let login = self.config.global.fingerprint == Some(req.fingerprint);

        if login {
            self.config.global.fingerprint = None;
            self.switch_wallet().await?;
        }

        let pool = self.connect_to_database(req.fingerprint).await?;

        query!(
            "
            DELETE FROM mempool_items;
            UPDATE blocks SET is_peak = FALSE WHERE is_peak = TRUE;
            "
        )
        .execute(&pool)
        .await?;

        if req.delete_coins {
            query!("DELETE FROM coins").execute(&pool).await?;
        }

        if req.delete_assets {
            query!(
                "
                DELETE FROM assets WHERE id != 0;
                DELETE FROM collections WHERE id != 0;
                "
            )
            .execute(&pool)
            .await?;
        }

        if req.delete_files {
            query!("DELETE FROM files").execute(&pool).await?;
        }

        if req.delete_offers {
            query!("DELETE FROM offers").execute(&pool).await?;
        }

        if req.delete_addresses {
            query!("DELETE FROM p2_puzzles").execute(&pool).await?;
        }

        if req.delete_blocks {
            query!("DELETE FROM blocks").execute(&pool).await?;
        }

        // reclaim disk space after all those deletes
        query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&pool)
            .await?;
        query("VACUUM").execute(&pool).await?;
        query("ANALYZE").execute(&pool).await?;

        if login {
            self.config.global.fingerprint = Some(req.fingerprint);
            self.save_config()?;
            self.switch_wallet().await?;
        }

        Ok(ResyncResponse {})
    }

    pub fn generate_mnemonic(&self, req: GenerateMnemonic) -> Result<GenerateMnemonicResponse> {
        let mut rng = ChaCha20Rng::from_entropy();
        let mnemonic = if req.use_24_words {
            let entropy: [u8; 32] = rng.r#gen();
            Mnemonic::from_entropy(&entropy)?
        } else {
            let entropy: [u8; 16] = rng.r#gen();
            Mnemonic::from_entropy(&entropy)?
        };
        Ok(GenerateMnemonicResponse {
            mnemonic: mnemonic.to_string(),
        })
    }

    pub async fn import_wallet(&mut self, req: ImportWallet) -> Result<ImportWalletResponse> {
        let ParsedKey {
            master_pk,
            master_sk,
            mnemonic,
        } = parse_key(&req.key)?;

        let fingerprint = master_pk.get_fingerprint();

        // A wallet whose key is missing from the keychain, such as after restoring an older
        // `keys.bin`, is hidden from the wallet list. Importing its key again restores it.
        let restoring = match self.wallet_entry(fingerprint) {
            Some(_) if self.keychain.contains(fingerprint) => {
                return Err(Error::WalletExists(fingerprint));
            }
            Some(_) => true,
            None => false,
        };

        // The keychain can hold keys that aren't wallets, so the key may already be there. It's
        // reused, unless this import adds secrets that the keychain doesn't have yet.
        self.store_key(
            &master_pk,
            master_sk.as_ref(),
            mnemonic.as_ref(),
            req.save_secrets,
        )?;

        if !restoring {
            self.wallet_config.wallets.push(Wallet {
                name: req.name,
                fingerprint,
                emoji: req.emoji,
                ..Default::default()
            });
        }

        self.config.global.fingerprint = Some(fingerprint);

        self.save_keychain()?;
        self.save_config()?;

        let pool = self.connect_to_database(fingerprint).await?;
        let db = Database::new(pool);

        let mut tx = db.tx().await?;

        if req.unhardened.unwrap_or(true) {
            let intermediate_unhardened_pk = master_to_wallet_unhardened_intermediate(&master_pk);

            for index in 0..req.derivation_index {
                let synthetic_key = intermediate_unhardened_pk
                    .derive_unhardened(index)
                    .derive_synthetic();
                let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();
                tx.insert_custody_p2_puzzle(
                    p2_puzzle_hash,
                    synthetic_key,
                    Derivation {
                        derivation_index: index,
                        is_hardened: false,
                    },
                )
                .await?;
            }
        }

        if req.hardened.unwrap_or(true)
            && let Some(master_sk) = master_sk
        {
            let intermediate_hardened_sk = master_to_wallet_hardened_intermediate(&master_sk);

            for index in 0..req.derivation_index {
                let synthetic_key = intermediate_hardened_sk
                    .derive_hardened(index)
                    .derive_synthetic()
                    .public_key();
                let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();
                tx.insert_custody_p2_puzzle(
                    p2_puzzle_hash,
                    synthetic_key,
                    Derivation {
                        derivation_index: index,
                        is_hardened: true,
                    },
                )
                .await?;
            }
        }

        tx.insert_arbor_p2_puzzle(master_pk).await?;

        tx.commit().await?;

        if req.login {
            self.switch_wallet().await?;
        }

        Ok(ImportWalletResponse { fingerprint })
    }

    pub fn delete_database(&mut self, req: DeleteDatabase) -> Result<DeleteDatabaseResponse> {
        let path = self.path.join("wallets").join(req.fingerprint.to_string());

        if path.try_exists()? {
            // Delete the specific SQLite file for this network
            let db_file = path.join(format!("{}.sqlite", req.network));
            if db_file.try_exists()? {
                fs::remove_file(&db_file)?;
            }
        }

        Ok(DeleteDatabaseResponse {})
    }

    pub fn delete_wallet(&mut self, req: DeleteWallet) -> Result<DeleteWalletResponse> {
        // Only a BLS wallet's key belongs to it. A key without a wallet isn't removed here.
        let owns_key = self
            .wallet_entry(req.fingerprint)
            .is_some_and(|wallet| matches!(wallet.kind, ConfigWalletKind::Bls));

        self.wallet_config
            .wallets
            .retain(|wallet| wallet.fingerprint != req.fingerprint);

        // Keys are kept if another wallet still uses them, or if the caller asked to keep them.
        if req.delete_keys && owns_key && !self.is_key_used(req.fingerprint) {
            self.keychain.remove(req.fingerprint);
        }

        if self.config.global.fingerprint == Some(req.fingerprint) {
            self.config.global.fingerprint = None;
        }

        self.save_keychain()?;
        self.save_config()?;

        let path = self.path.join("wallets").join(req.fingerprint.to_string());
        if path.try_exists()? {
            fs::remove_dir_all(path)?;
        }

        Ok(DeleteWalletResponse {})
    }

    pub fn rename_wallet(&mut self, req: RenameWallet) -> Result<RenameWalletResponse> {
        let Some(wallet) = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|wallet| wallet.fingerprint == req.fingerprint)
        else {
            return Err(Error::UnknownFingerprint);
        };

        wallet.name = req.name;
        self.save_config()?;

        Ok(RenameWalletResponse {})
    }

    pub fn set_wallet_emoji(&mut self, req: SetWalletEmoji) -> Result<SetWalletEmojiResponse> {
        let Some(wallet) = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|wallet| wallet.fingerprint == req.fingerprint)
        else {
            return Err(Error::UnknownFingerprint);
        };

        wallet.emoji = req.emoji;
        self.save_config()?;

        Ok(SetWalletEmojiResponse {})
    }

    pub fn get_wallet(&self, req: GetWallet) -> Result<GetWalletResponse> {
        let fingerprint = req.fingerprint.or(self.config.global.fingerprint);

        let Some(fingerprint) = fingerprint else {
            return Ok(GetWalletResponse { wallet: None });
        };

        // Only fingerprints listed in `wallets.toml` are wallets.
        let Some(wallet) = self.wallet_entry(fingerprint) else {
            return Ok(GetWalletResponse { wallet: None });
        };

        Ok(GetWalletResponse {
            wallet: self.wallet_info(wallet),
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

    pub fn get_wallets(&self, _req: GetWallets) -> Result<GetWalletsResponse> {
        let wallets = self
            .wallet_config
            .wallets
            .iter()
            .filter_map(|wallet| self.wallet_info(wallet))
            .collect();

        Ok(GetWalletsResponse { wallets })
    }

    pub fn get_key(&self, req: GetKey) -> Result<GetKeyResponse> {
        Ok(GetKeyResponse {
            key: self.key_info(req.fingerprint)?,
        })
    }

    pub fn get_keys(&self, _req: GetKeys) -> Result<GetKeysResponse> {
        let mut fingerprints: Vec<u32> = self.keychain.fingerprints().collect();
        fingerprints.sort_unstable();

        let mut keys = Vec::new();

        for fingerprint in fingerprints {
            if let Some(key) = self.key_info(fingerprint)? {
                keys.push(key);
            }
        }

        Ok(GetKeysResponse { keys })
    }

    fn key_info(&self, fingerprint: u32) -> Result<Option<KeyInfo>> {
        let Some(master_pk) = self.keychain.extract_public_key(fingerprint)? else {
            return Ok(None);
        };

        Ok(Some(KeyInfo {
            fingerprint,
            kind: KeyKind::Bls,
            public_key: hex::encode(master_pk.to_bytes()),
            has_secrets: self.keychain.has_secret_key(fingerprint),
        }))
    }

    /// Describes a wallet. Returns `None` for a BLS wallet whose key is missing from the
    /// keychain.
    fn wallet_info(&self, wallet: &Wallet) -> Option<WalletInfo> {
        let kind = match wallet.kind {
            ConfigWalletKind::Bls => {
                if !self.keychain.contains(wallet.fingerprint) {
                    return None;
                }

                ApiWalletKind::Bls
            }
        };

        Some(WalletInfo {
            name: wallet.name.clone(),
            fingerprint: wallet.fingerprint,
            kind,
            network_id: wallet.network.clone().unwrap_or_else(|| self.network_id()),
            emoji: wallet.emoji.clone(),
        })
    }

    /// Adds a BLS key to the keychain, and returns whether the keychain changed. A key that's
    /// already there is kept, unless this adds secrets that it doesn't have yet.
    fn store_key(
        &mut self,
        master_pk: &PublicKey,
        master_sk: Option<&SecretKey>,
        mnemonic: Option<&Mnemonic>,
        save_secrets: bool,
    ) -> Result<bool> {
        let fingerprint = master_pk.get_fingerprint();
        let store_secrets = save_secrets && master_sk.is_some();

        if self.keychain.contains(fingerprint)
            && (!store_secrets || self.keychain.has_secret_key(fingerprint))
        {
            return Ok(false);
        }

        self.keychain.remove(fingerprint);

        match (mnemonic, master_sk) {
            (Some(mnemonic), _) if store_secrets => {
                self.keychain.add_mnemonic(mnemonic, b"")?;
            }
            (None, Some(master_sk)) if store_secrets => {
                self.keychain.add_secret_key(master_sk, b"")?;
            }
            _ => {
                self.keychain.add_public_key(master_pk)?;
            }
        }

        Ok(true)
    }

    /// Adds a BLS key to the keychain without creating a wallet for it.
    pub fn add_key(&mut self, req: AddKey) -> Result<AddKeyResponse> {
        let ParsedKey {
            master_pk,
            master_sk,
            mnemonic,
        } = parse_key(&req.key)?;

        if !self.store_key(&master_pk, master_sk.as_ref(), mnemonic.as_ref(), true)? {
            return Err(KeychainError::KeyExists.into());
        }

        self.save_keychain()?;

        Ok(AddKeyResponse {
            fingerprint: master_pk.get_fingerprint(),
        })
    }

    /// Removes a key from the keychain. A key that a wallet uses can only be removed along with
    /// the wallet, since the wallet can't be used without it.
    pub fn delete_key(&mut self, req: DeleteKey) -> Result<DeleteKeyResponse> {
        if self.is_key_used(req.fingerprint) {
            return Err(Error::KeyInUse(req.fingerprint));
        }

        if !self.keychain.remove(req.fingerprint) {
            return Err(Error::UnknownFingerprint);
        }

        self.save_keychain()?;

        Ok(DeleteKeyResponse {})
    }

    pub async fn get_wallet_address(
        &self,
        req: GetWalletAddress,
    ) -> Result<GetWalletAddressResponse> {
        let WalletKind::Bls { intermediate_pk } = self.wallet_kind(req.fingerprint)?;

        // Return the change_address override directly if one is configured
        let wallet_cfg = self
            .wallet_config
            .wallets
            .iter()
            .find(|w| w.fingerprint == req.fingerprint);

        if let Some(cfg) = wallet_cfg
            && let Some(change_address) = &cfg.change_address
        {
            return Ok(GetWalletAddressResponse {
                address: change_address.clone(),
            });
        }

        let network = self
            .network_list
            .by_name(&req.network_id)
            .ok_or(Error::UnknownFingerprint)?;

        let prefix = network.prefix();
        // Try to read the current receive address from the wallet's DB
        let p2_puzzle_hash = self
            .address_from_db(req.fingerprint, &req.network_id, false)
            .await
            .ok()
            .flatten();

        // Fall back to deriving index 0 from the master public key
        let p2_puzzle_hash = p2_puzzle_hash.unwrap_or_else(|| {
            let synthetic_key = intermediate_pk.derive_unhardened(0).derive_synthetic();
            StandardArgs::curry_tree_hash(synthetic_key).into()
        });

        let address = Address::new(p2_puzzle_hash, prefix).encode()?;

        Ok(GetWalletAddressResponse { address })
    }

    async fn address_from_db(
        &self,
        fingerprint: u32,
        network_id: &str,
        hardened: bool,
    ) -> Result<Option<Bytes32>> {
        let db_path = self
            .path
            .join("wallets")
            .join(fingerprint.to_string())
            .join(format!("{network_id}.sqlite"));

        if !db_path.try_exists().unwrap_or(false) {
            return Ok(None);
        }

        let pool = self.connect_to_pool(db_path).await?;
        let db = Database::new(pool);
        let mut tx = db.read_tx().await?;
        let index = tx.unused_derivation_index(hardened).await?;
        Ok(tx.custody_p2_puzzle_hash(index, hardened).await.ok())
    }
}

/// A BLS key given as a mnemonic, or a hex encoded secret or public key.
struct ParsedKey {
    master_pk: PublicKey,
    master_sk: Option<SecretKey>,
    mnemonic: Option<Mnemonic>,
}

fn parse_key(key: &str) -> Result<ParsedKey> {
    let mut key_hex = key;

    if key_hex.starts_with("0x") || key_hex.starts_with("0X") {
        key_hex = &key_hex[2..];
    }

    let (master_pk, master_sk, mnemonic) = if let Ok(bytes) = hex::decode(key_hex) {
        if let Ok(master_pk) = bytes.clone().try_into() {
            (PublicKey::from_bytes(&master_pk)?, None, None)
        } else if let Ok(master_sk) = bytes.try_into() {
            let master_sk = SecretKey::from_bytes(&master_sk)?;
            (master_sk.public_key(), Some(master_sk), None)
        } else {
            return Err(Error::InvalidKey);
        }
    } else {
        let words: Vec<&str> = key.split_whitespace().collect();
        let word_count = words.len();

        if word_count != 12 && word_count != 24 {
            return Err(Error::InvalidMnemonic(format!(
                "Expected 12 or 24 words, but got {word_count}."
            )));
        }

        let mnemonic = Mnemonic::from_str(key).map_err(|e| match e {
            bip39::Error::BadWordCount(count) => {
                Error::InvalidMnemonic(format!("Expected 12 or 24 words, but got {count}."))
            }
            bip39::Error::UnknownWord(idx) => Error::InvalidMnemonic(format!(
                "Word #{} ({}) is not a valid BIP39 word.",
                idx + 1,
                words.get(idx).copied().unwrap_or("unknown"),
            )),
            bip39::Error::InvalidChecksum => Error::InvalidMnemonic(
                "Invalid checksum. Please verify all words are correct and in the right order."
                    .to_string(),
            ),
            _ => Error::InvalidMnemonic(format!("Invalid mnemonic: {e}")),
        })?;
        let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        (master_sk.public_key(), Some(master_sk), Some(mnemonic))
    };

    Ok(ParsedKey {
        master_pk,
        master_sk,
        mnemonic,
    })
}
