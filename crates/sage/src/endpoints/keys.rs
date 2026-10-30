use std::fs;

use bip39::Mnemonic;
use chia_wallet_sdk::{
    chia::{
        bls::{
            DerivableKey, master_to_wallet_hardened_intermediate,
            master_to_wallet_unhardened_intermediate,
        },
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
        sha2::Sha256,
    },
    prelude::*,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{
    AddKey, AddKeyResponse, AddWatchAddresses, AddWatchAddressesResponse, DeleteDatabase,
    DeleteDatabaseResponse, DeleteKey, DeleteKeyResponse, DeleteWallet, DeleteWalletResponse,
    GenerateMnemonic, GenerateMnemonicResponse, GetKey, GetKeyResponse, GetKeys, GetKeysResponse,
    GetSecretKey, GetSecretKeyResponse, GetWallet, GetWalletAddress, GetWalletAddressResponse,
    GetWalletResponse, GetWallets, GetWalletsResponse, ImportWallet, ImportWalletResponse,
    ImportWatchWallet, ImportWatchWalletResponse, KeyInfo, KeyKind, Login, LoginResponse, Logout,
    LogoutResponse, RemoveWatchAddresses, RemoveWatchAddressesResponse, RenameWallet,
    RenameWalletResponse, Resync, ResyncResponse, SecretKeyInfo, SetWalletEmoji,
    SetWalletEmojiResponse, WalletInfo, WalletKind as ApiWalletKind,
};
use sage_config::{Wallet, WalletKind as ConfigWalletKind};
use sage_database::{Database, Derivation};
use sage_wallet::{SyncCommand, WalletKind, p2_singleton_puzzle_hash};
use sqlx::query;

use crate::{
    Error, ParsedKey, Result, Sage, assign_fingerprint, encode_vault_id, parse_key,
    watch_p2_puzzles,
};

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
        let restoring = match self.wallet_entry(fingerprint).map(|wallet| &wallet.kind) {
            Some(ConfigWalletKind::Watch { .. } | ConfigWalletKind::Vault { .. }) => {
                return Err(Error::FingerprintCollision(fingerprint));
            }
            Some(ConfigWalletKind::Bls) if self.keychain.contains(fingerprint) => {
                return Err(Error::WalletExists(fingerprint));
            }
            Some(ConfigWalletKind::Bls) => true,
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
        // A BLS wallet's key belongs to it, and a vault's signers' keys belong to the vault. A key
        // without a wallet isn't removed here.
        let owned_keys = match self
            .wallet_entry(req.fingerprint)
            .map(|wallet| &wallet.kind)
        {
            Some(ConfigWalletKind::Bls) => vec![req.fingerprint],
            Some(ConfigWalletKind::Vault { signers, .. }) => signers.clone(),
            Some(ConfigWalletKind::Watch { .. }) | None => Vec::new(),
        };

        self.wallet_config
            .wallets
            .retain(|wallet| wallet.fingerprint != req.fingerprint);

        // Keys are kept if another wallet still uses them, or if the caller asked to keep them.
        if req.delete_keys {
            for fingerprint in owned_keys {
                if !self.is_key_used(fingerprint) {
                    self.keychain.remove(fingerprint);
                }
            }
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
            wallet: self.wallet_info(wallet)?,
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
        let mut wallets = Vec::new();

        for wallet in &self.wallet_config.wallets {
            if let Some(info) = self.wallet_info(wallet)? {
                wallets.push(info);
            }
        }

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
    fn wallet_info(&self, wallet: &Wallet) -> Result<Option<WalletInfo>> {
        let network_id = wallet.network.clone().unwrap_or_else(|| self.network_id());

        let kind = match &wallet.kind {
            ConfigWalletKind::Bls => {
                if !self.keychain.contains(wallet.fingerprint) {
                    return Ok(None);
                }

                ApiWalletKind::Bls
            }
            ConfigWalletKind::Watch { p2_puzzle_hashes } => {
                let prefix = self.network_prefix(&network_id);

                ApiWalletKind::Watch {
                    addresses: p2_puzzle_hashes
                        .iter()
                        .map(|&p2_puzzle_hash| {
                            Ok(Address::new(p2_puzzle_hash, prefix.clone()).encode()?)
                        })
                        .collect::<Result<_>>()?,
                }
            }
            ConfigWalletKind::Vault {
                launcher_id,
                signers,
            } => ApiWalletKind::Vault {
                launcher_id: encode_vault_id(*launcher_id)?,
                signers: signers.clone(),
            },
        };

        Ok(Some(WalletInfo {
            name: wallet.name.clone(),
            fingerprint: wallet.fingerprint,
            kind,
            network_id,
            emoji: wallet.emoji.clone(),
        }))
    }

    /// The address prefix of a network, falling back to the active network's.
    fn network_prefix(&self, network_id: &str) -> String {
        self.network_list
            .by_name(network_id)
            .unwrap_or_else(|| self.network())
            .prefix()
    }

    pub async fn import_watch_wallet(
        &mut self,
        req: ImportWatchWallet,
    ) -> Result<ImportWatchWalletResponse> {
        let p2_puzzle_hashes = parse_watch_addresses(req.addresses)?;

        if p2_puzzle_hashes.is_empty() {
            return Err(Error::NoWatchAddresses);
        }

        let sorted = sorted_set(&p2_puzzle_hashes);

        if let Some(existing) = self.wallet_config.wallets.iter().find(|wallet| {
            matches!(&wallet.kind, ConfigWalletKind::Watch { p2_puzzle_hashes } if sorted_set(p2_puzzle_hashes) == sorted)
        }) {
            return Err(Error::WalletExists(existing.fingerprint));
        }

        // A watch wallet has no key to take a fingerprint from, so it's hashed from the set of
        // addresses it's imported with. It stays the same when addresses are added or removed.
        let mut hasher = Sha256::new();
        for p2_puzzle_hash in &sorted {
            hasher.update(p2_puzzle_hash);
        }
        let seed = hasher.finalize();

        let fingerprint =
            assign_fingerprint(&seed, |fingerprint| self.fingerprint_taken(fingerprint))
                .ok_or(Error::NoFreeFingerprint)?;

        self.wallet_config.wallets.push(Wallet {
            name: req.name,
            fingerprint,
            emoji: req.emoji,
            kind: ConfigWalletKind::Watch { p2_puzzle_hashes },
            ..Default::default()
        });

        if req.login {
            self.config.global.fingerprint = Some(fingerprint);
        }

        self.save_config()?;

        if req.login {
            self.switch_wallet().await?;
        }

        Ok(ImportWatchWalletResponse { fingerprint })
    }

    pub async fn add_watch_addresses(
        &mut self,
        req: AddWatchAddresses,
    ) -> Result<AddWatchAddressesResponse> {
        let new = parse_watch_addresses(req.addresses)?;

        let p2_puzzle_hashes = self.watch_wallet_mut(req.fingerprint)?;

        for p2_puzzle_hash in new {
            if !p2_puzzle_hashes.contains(&p2_puzzle_hash) {
                p2_puzzle_hashes.push(p2_puzzle_hash);
            }
        }

        self.save_config()?;
        self.update_watched_p2_puzzles(req.fingerprint).await?;

        Ok(AddWatchAddressesResponse {})
    }

    pub async fn remove_watch_addresses(
        &mut self,
        req: RemoveWatchAddresses,
    ) -> Result<RemoveWatchAddressesResponse> {
        let removed = parse_watch_addresses(req.addresses)?;

        let p2_puzzle_hashes = self.watch_wallet_mut(req.fingerprint)?;

        let remaining: Vec<Bytes32> = p2_puzzle_hashes
            .iter()
            .copied()
            .filter(|p2_puzzle_hash| !removed.contains(p2_puzzle_hash))
            .collect();

        if remaining.is_empty() {
            return Err(Error::NoWatchAddresses);
        }

        *p2_puzzle_hashes = remaining;

        self.save_config()?;
        self.update_watched_p2_puzzles(req.fingerprint).await?;

        Ok(RemoveWatchAddressesResponse {})
    }

    fn watch_wallet_mut(&mut self, fingerprint: u32) -> Result<&mut Vec<Bytes32>> {
        let wallet = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|wallet| wallet.fingerprint == fingerprint)
            .ok_or(Error::UnknownFingerprint)?;

        match &mut wallet.kind {
            ConfigWalletKind::Watch { p2_puzzle_hashes } => Ok(p2_puzzle_hashes),
            ConfigWalletKind::Bls | ConfigWalletKind::Vault { .. } => {
                Err(Error::NotWatchWallet(fingerprint))
            }
        }
    }

    /// Brings the open watch wallet's database in line with `wallets.toml`, and subscribes to
    /// newly watched puzzle hashes, so they sync without a full resync. Other wallets catch up
    /// the next time they're opened.
    async fn update_watched_p2_puzzles(&self, fingerprint: u32) -> Result<()> {
        let Some(wallet) = self
            .wallet
            .as_ref()
            .filter(|wallet| wallet.fingerprint == fingerprint)
        else {
            return Ok(());
        };

        let p2_puzzle_hashes = self
            .watched_p2_puzzle_hashes(fingerprint)
            .ok_or(Error::NotWatchWallet(fingerprint))?;

        let added = watch_p2_puzzles(&wallet.db, p2_puzzle_hashes).await?;

        if !added.is_empty() {
            self.command_sender
                .send(SyncCommand::SubscribePuzzles {
                    puzzle_hashes: added,
                })
                .await?;
        }

        Ok(())
    }

    /// Whether a new wallet without a BLS key can't use this fingerprint. Besides existing
    /// wallets, this avoids keys in the keychain, whose wallets would use their fingerprint,
    /// and databases left behind by wallets that are no longer listed.
    pub(crate) fn fingerprint_taken(&self, fingerprint: u32) -> bool {
        self.wallet_entry(fingerprint).is_some()
            || self.keychain.contains(fingerprint)
            || self
                .path
                .join("wallets")
                .join(fingerprint.to_string())
                .exists()
    }

    /// Adds a BLS key to the keychain, and returns whether the keychain changed. A key that's
    /// already there is kept, unless this adds a secret key or mnemonic that it doesn't have.
    pub(crate) fn store_key(
        &mut self,
        master_pk: &PublicKey,
        master_sk: Option<&SecretKey>,
        mnemonic: Option<&Mnemonic>,
        save_secrets: bool,
    ) -> Result<bool> {
        let fingerprint = master_pk.get_fingerprint();
        let store_secrets = save_secrets && master_sk.is_some();

        // Fingerprints are only 32 bits, so a different key can have the same one. Replacing the
        // stored key with it would silently change the wallet that uses it.
        let existing = match self.keychain.extract_public_key(fingerprint)? {
            Some(existing_pk) if existing_pk != *master_pk => {
                return Err(Error::KeyConflict(fingerprint));
            }
            Some(_) => {
                let (existing_mnemonic, existing_sk) =
                    self.keychain.extract_secrets(fingerprint, b"")?;

                let adds_secrets = store_secrets
                    && (existing_sk.is_none()
                        || (mnemonic.is_some() && existing_mnemonic.is_none()));

                if !adds_secrets {
                    return Ok(false);
                }

                Some(existing_sk)
            }
            None => None,
        };

        self.keychain.remove(fingerprint);

        let stored = match (mnemonic, master_sk) {
            (Some(mnemonic), _) if store_secrets => self.keychain.add_mnemonic(mnemonic, b""),
            (None, Some(master_sk)) if store_secrets => {
                self.keychain.add_secret_key(master_sk, b"")
            }
            _ => self.keychain.add_public_key(master_pk),
        };

        // If the new entry can't be stored, the old one is put back rather than lost.
        if let Err(error) = stored {
            match existing {
                Some(Some(existing_sk)) => {
                    self.keychain.add_secret_key(&existing_sk, b"")?;
                }
                Some(None) => {
                    self.keychain.add_public_key(master_pk)?;
                }
                None => {}
            }

            return Err(error.into());
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
            return Err(Error::KeyExists(master_pk.get_fingerprint()));
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
        let kind = self.wallet_kind(req.fingerprint)?;

        let network = self
            .network_list
            .by_name(&req.network_id)
            .ok_or(Error::UnknownFingerprint)?;

        let prefix = network.prefix();

        // A vault receives at its first address, which never changes.
        if let WalletKind::Vault { launcher_id } = kind {
            return Ok(GetWalletAddressResponse {
                address: Address::new(p2_singleton_puzzle_hash(launcher_id, 0), prefix).encode()?,
            });
        }

        // Watch wallets don't make change, so a change address doesn't replace their first
        // watched address.
        let WalletKind::Bls { intermediate_pk } = kind else {
            let p2_puzzle_hash = self
                .watched_p2_puzzle_hashes(req.fingerprint)
                .and_then(|p2_puzzle_hashes| p2_puzzle_hashes.first().copied())
                .ok_or(Error::NoWatchAddresses)?;

            return Ok(GetWalletAddressResponse {
                address: Address::new(p2_puzzle_hash, prefix).encode()?,
            });
        };

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

fn sorted_set(p2_puzzle_hashes: &[Bytes32]) -> Vec<Bytes32> {
    let mut sorted = p2_puzzle_hashes.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

/// Decodes addresses for a watch wallet, without duplicates and in the order given. Their prefix
/// isn't checked: only the puzzle hashes are stored, and the wallet's network decides how they're
/// shown, so addresses for another network can be imported before switching the wallet to it.
fn parse_watch_addresses(addresses: Vec<String>) -> Result<Vec<Bytes32>> {
    let mut p2_puzzle_hashes = Vec::new();

    for address in addresses {
        let address = Address::decode(address.trim())?;

        if !p2_puzzle_hashes.contains(&address.puzzle_hash) {
            p2_puzzle_hashes.push(address.puzzle_hash);
        }
    }

    Ok(p2_puzzle_hashes)
}
