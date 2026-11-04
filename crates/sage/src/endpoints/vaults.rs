use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sage_api::{
    AttachVaultSigner, AttachVaultSignerResponse, FindVault, FindVaultResponse, GetVault,
    GetVaultResponse, ImportVault, ImportVaultResponse, RebuildVaultConfig,
    RebuildVaultConfigResponse, VaultConfigRecord, VaultConfigSource as ApiVaultConfigSource,
    VaultMemberKind, VaultMemberRecord, VaultPuzzleVersion, VaultRecord, VaultRecoveryKey,
    VaultSignerRecord, VaultSignersRecord, VaultState as ApiVaultState,
};
use sage_config::{Wallet as WalletEntry, WalletKind as ConfigWalletKind};
use sage_database::{
    Database, VaultConfig, VaultConfigRow, VaultConfigSource, VaultLayout, VaultMember,
    VaultSigners, VaultState,
};
use sage_wallet::{
    RecoveryKey, VaultAddressLookup, WalletKind, common_recovery_timelocks,
    fetch_latest_vault_coin, find_vault_by_address, rebuild_vault_configs, recovery_key_members,
    track_vault,
};

use crate::{
    Error, ParsedKey, Result, Sage, assign_fingerprint, encode_vault_id, parse_key, parse_vault_id,
};

impl Sage {
    pub async fn import_vault(&mut self, req: ImportVault) -> Result<ImportVaultResponse> {
        let launcher_id = match (req.launcher_id, req.address) {
            (Some(launcher_id), None) => parse_vault_id(launcher_id)?,
            (None, Some(address)) => {
                let p2_puzzle_hash = self.parse_address(address)?;
                self.find_vault_by_address(p2_puzzle_hash).await?
            }
            _ => return Err(Error::VaultImportSource),
        };

        if let Some(existing) = self.wallet_config.wallets.iter().find(|wallet| {
            matches!(wallet.kind, ConfigWalletKind::Vault { launcher_id: existing, .. } if existing == launcher_id)
        }) {
            return Err(Error::WalletExists(existing.fingerprint));
        }

        // The vault must exist, and a recovery key must rebuild its config from its spends.
        let configs = if let Some(recovery) = req.recovery {
            self.rebuild_configs(launcher_id, recovery).await?
        } else {
            self.latest_vault_coin(launcher_id).await?;
            Vec::new()
        };

        // A vault has no key to take a fingerprint from, so it's taken from its launcher id.
        let fingerprint = assign_fingerprint(&launcher_id, |fingerprint| {
            self.fingerprint_taken(fingerprint)
        })
        .ok_or(Error::NoFreeFingerprint)?;

        self.wallet_config.wallets.push(WalletEntry {
            name: req.name,
            fingerprint,
            emoji: req.emoji,
            kind: ConfigWalletKind::Vault {
                launcher_id,
                signers: Vec::new(),
            },
            ..Default::default()
        });

        if req.login {
            self.config.global.fingerprint = Some(fingerprint);
        }

        self.save_config()?;

        let db = Database::new(self.connect_to_database(fingerprint).await?);
        let mut tx = db.tx().await?;
        track_vault(&mut tx, launcher_id).await?;
        for config in &configs {
            tx.insert_vault_config(config).await?;
        }
        tx.commit().await?;

        if req.login {
            self.switch_wallet().await?;
        }

        Ok(ImportVaultResponse {
            fingerprint,
            launcher_id: encode_vault_id(launcher_id)?,
        })
    }

    pub async fn get_vault(&self, _req: GetVault) -> Result<GetVaultResponse> {
        let wallet = self.wallet()?;
        let launcher_id = self.vault_launcher_id(wallet.fingerprint)?;
        let signers = self.vault_signer_keys(wallet.fingerprint)?;

        let status = wallet.db.vault_status(launcher_id).await?;
        let pre_recovery = latest_custody_config(&wallet.db, launcher_id).await?;

        let (state, recovery_started_at) = match status.as_ref().map(|status| status.state) {
            None | Some(VaultState::Unsynced) => (ApiVaultState::Unsynced, None),
            Some(VaultState::Active) => (ApiVaultState::Active, None),
            Some(VaultState::RecoveryPending { started_at }) => {
                (ApiVaultState::RecoveryPending, started_at)
            }
            Some(VaultState::Melted) => (ApiVaultState::Melted, None),
        };

        let coin = status.as_ref().and_then(|status| status.coin);
        let config = status.and_then(|status| status.config);

        let signer_records = signers
            .iter()
            .map(|&(fingerprint, public_key)| {
                let (custody, recovery) =
                    member_roles(config.as_ref(), pre_recovery.as_ref(), public_key);

                VaultSignerRecord {
                    fingerprint,
                    custody,
                    recovery,
                }
            })
            .collect();

        Ok(GetVaultResponse {
            vault: VaultRecord {
                launcher_id: encode_vault_id(launcher_id)?,
                state,
                recovery_started_at,
                coin_id: coin.map(|row| hex::encode(row.coin.coin_id())),
                custody_hash: coin.and_then(|row| row.custody_hash).map(hex::encode),
                config: config
                    .map(|config| config_record(&config, pre_recovery.as_ref(), &signers))
                    .transpose()?,
                signers: signer_records,
            },
        })
    }

    pub async fn attach_vault_signer(
        &mut self,
        req: AttachVaultSigner,
    ) -> Result<AttachVaultSignerResponse> {
        let launcher_id = self.vault_launcher_id(req.fingerprint)?;

        let ParsedKey {
            master_pk,
            master_sk,
            mnemonic,
        } = parse_key(&req.key)?;

        // A signer must be able to sign.
        if master_sk.is_none() {
            return Err(Error::NoSigningKey);
        }

        let db = self.vault_database(req.fingerprint).await?;
        let config = db
            .vault_status(launcher_id)
            .await?
            .and_then(|status| status.config);
        let pre_recovery = latest_custody_config(&db, launcher_id).await?;

        if config.is_none() && pre_recovery.is_none() {
            return Err(Error::VaultConfigUnknown);
        }

        let key_fingerprint = master_pk.get_fingerprint();
        let (custody, recovery) = member_roles(config.as_ref(), pre_recovery.as_ref(), master_pk);

        if !custody && !recovery {
            return Err(Error::NotVaultMember(key_fingerprint));
        }

        self.store_key(&master_pk, master_sk.as_ref(), mnemonic.as_ref(), true)?;

        if let Some(ConfigWalletKind::Vault { signers, .. }) = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|wallet| wallet.fingerprint == req.fingerprint)
            .map(|wallet| &mut wallet.kind)
            && !signers.contains(&key_fingerprint)
        {
            signers.push(key_fingerprint);
        }

        self.save_keychain()?;
        self.save_config()?;

        Ok(AttachVaultSignerResponse {
            fingerprint: key_fingerprint,
            custody,
            recovery,
        })
    }

    pub async fn rebuild_vault_config(
        &self,
        req: RebuildVaultConfig,
    ) -> Result<RebuildVaultConfigResponse> {
        let wallet = self.wallet()?;
        let launcher_id = self.vault_launcher_id(wallet.fingerprint)?;

        let configs = self.rebuild_configs(launcher_id, req.recovery).await?;

        let mut tx = wallet.db.tx().await?;
        for config in &configs {
            tx.insert_vault_config(config).await?;
        }
        tx.commit().await?;

        // The latest config is the vault's current custody, or the closest to it.
        let recovery_timelock = configs
            .first()
            .map(|config| match &config.layout {
                VaultLayout::Custody(config) => config.recovery_timelock,
                VaultLayout::Recovery(config) => config.recovery_timelock,
            })
            .ok_or(Error::VaultConfigUnknown)?;

        Ok(RebuildVaultConfigResponse { recovery_timelock })
    }

    pub async fn find_vault(&self, req: FindVault) -> Result<FindVaultResponse> {
        let p2_puzzle_hash = Address::decode(req.address.trim())?.puzzle_hash;

        let launcher_id = match self.find_vault_by_address(p2_puzzle_hash).await {
            Ok(launcher_id) => Some(encode_vault_id(launcher_id)?),
            Err(Error::VaultAddressNeverSpent | Error::NotVaultAddress) => None,
            Err(error) => return Err(error),
        };

        Ok(FindVaultResponse { launcher_id })
    }

    /// The launcher id of a vault wallet.
    fn vault_launcher_id(&self, fingerprint: u32) -> Result<Bytes32> {
        match self.wallet_kind(fingerprint)? {
            WalletKind::Vault { launcher_id } => Ok(launcher_id),
            WalletKind::Bls { .. } | WalletKind::Watch => Err(Error::NotVaultWallet(fingerprint)),
        }
    }

    /// The fingerprints and public keys of the keychain keys which sign for a vault wallet.
    fn vault_signer_keys(&self, fingerprint: u32) -> Result<Vec<(u32, PublicKey)>> {
        let Some(ConfigWalletKind::Vault { signers, .. }) =
            self.wallet_entry(fingerprint).map(|wallet| &wallet.kind)
        else {
            return Err(Error::NotVaultWallet(fingerprint));
        };

        let mut keys = Vec::new();

        for &signer in signers {
            if let Some(public_key) = self.keychain.extract_public_key(signer)? {
                keys.push((signer, public_key));
            }
        }

        Ok(keys)
    }

    /// The database of a vault wallet, which doesn't need to be open.
    async fn vault_database(&self, fingerprint: u32) -> Result<Database> {
        if let Some(wallet) = self
            .wallet
            .as_ref()
            .filter(|wallet| wallet.fingerprint == fingerprint)
        {
            return Ok(wallet.db.clone());
        }

        Ok(Database::new(self.connect_to_database(fingerprint).await?))
    }

    /// Rebuilds a vault's configs from its spends and its recovery key.
    async fn rebuild_configs(
        &self,
        launcher_id: Bytes32,
        recovery: VaultRecoveryKey,
    ) -> Result<Vec<VaultConfigRow>> {
        let members = recovery_key_members(parse_recovery_key(&recovery.key)?);

        if members.is_empty() {
            return Err(Error::InvalidRecoveryKey);
        }

        let timelocks = match recovery.timelock {
            Some(0) => return Err(Error::InvalidRecoveryTimelock),
            Some(timelock) => vec![timelock],
            None => common_recovery_timelocks(),
        };

        let peer = self
            .peer_state
            .lock()
            .await
            .acquire_peer()
            .ok_or(Error::NoPeers)?;

        Ok(rebuild_vault_configs(
            &peer,
            launcher_id,
            self.network().genesis_challenge,
            members,
            timelocks,
        )
        .await?)
    }

    async fn latest_vault_coin(&self, launcher_id: Bytes32) -> Result<CoinState> {
        let peer = self
            .peer_state
            .lock()
            .await
            .acquire_peer()
            .ok_or(Error::NoPeers)?;

        match fetch_latest_vault_coin(&peer, launcher_id, self.network().genesis_challenge).await? {
            Some(coin_state) => Ok(coin_state),
            None => Err(Error::VaultNotFound(encode_vault_id(launcher_id)?)),
        }
    }

    async fn find_vault_by_address(&self, p2_puzzle_hash: Bytes32) -> Result<Bytes32> {
        let peer = self
            .peer_state
            .lock()
            .await
            .acquire_peer()
            .ok_or(Error::NoPeers)?;

        match find_vault_by_address(&peer, p2_puzzle_hash, self.network().genesis_challenge).await?
        {
            VaultAddressLookup::Found(p2_singleton) => Ok(p2_singleton.launcher_id),
            VaultAddressLookup::NeverSpent => Err(Error::VaultAddressNeverSpent),
            VaultAddressLookup::NotVault => Err(Error::NotVaultAddress),
        }
    }
}

/// Parses a vault's recovery key: a public key in hex, or the mnemonic or secret key of a BLS key,
/// whose master public key is the member, as in Cloud Wallet.
fn parse_recovery_key(key: &str) -> Result<RecoveryKey> {
    let key = key.trim();
    let stripped = key
        .strip_prefix("0x")
        .or_else(|| key.strip_prefix("0X"))
        .unwrap_or(key);

    if let Ok(bytes) = hex::decode(stripped)
        && let Ok(bytes) = <[u8; 33]>::try_from(bytes.as_slice())
    {
        return Ok(RecoveryKey::Secp(bytes));
    }

    let ParsedKey { master_pk, .. } = parse_key(key).map_err(|_| Error::InvalidRecoveryKey)?;

    Ok(RecoveryKey::Bls(master_pk))
}

/// The config of the vault's latest coin which isn't in the recovery state. While recovery is
/// pending, this is the config from before it started, whose custody can cancel it, and whose
/// recovery members started it.
async fn latest_custody_config(db: &Database, launcher_id: Bytes32) -> Result<Option<VaultConfig>> {
    // The current coin's config is matched by its puzzle hash if its custody hash isn't known.
    if let Some(VaultConfigRow {
        layout: VaultLayout::Custody(config),
        ..
    }) = db
        .vault_status(launcher_id)
        .await?
        .and_then(|status| status.config)
    {
        return Ok(Some(config));
    }

    for row in db.vault_coins(launcher_id).await?.into_iter().rev() {
        let Some(custody_hash) = row.custody_hash else {
            continue;
        };

        if let Some(VaultConfigRow {
            layout: VaultLayout::Custody(config),
            ..
        }) = db.vault_config(custody_hash).await?
        {
            return Ok(Some(config));
        }
    }

    Ok(None)
}

fn is_member(signers: &VaultSigners, public_key: PublicKey) -> bool {
    signers.members.iter().any(|member| {
        matches!(member, VaultMember::Bls { public_key: member, .. } if *member == public_key)
    })
}

/// Whether a key is a custody member of the vault's current config, and whether it's a recovery
/// member of its latest config outside of the recovery state.
fn member_roles(
    config: Option<&VaultConfigRow>,
    pre_recovery: Option<&VaultConfig>,
    public_key: PublicKey,
) -> (bool, bool) {
    let custody = match config.map(|config| &config.layout) {
        Some(VaultLayout::Custody(config)) => is_member(&config.custody, public_key),
        Some(VaultLayout::Recovery(config)) => is_member(&config.custody, public_key),
        None => pre_recovery.is_some_and(|config| is_member(&config.custody, public_key)),
    };

    let recovery = pre_recovery.is_some_and(|config| is_member(&config.recovery, public_key));

    (custody, recovery)
}

fn config_record(
    config: &VaultConfigRow,
    pre_recovery: Option<&VaultConfig>,
    signers: &[(u32, PublicKey)],
) -> Result<VaultConfigRecord> {
    let (custody, recovery, recovery_timelock, version, recovery_custody_hash) =
        match &config.layout {
            VaultLayout::Custody(config) => (
                &config.custody,
                Some(&config.recovery),
                config.recovery_timelock,
                config.version,
                None,
            ),
            VaultLayout::Recovery(config) => (
                &config.custody,
                pre_recovery.map(|config| &config.recovery),
                config.recovery_timelock,
                config.version,
                Some(hex::encode(config.recovery_custody_hash)),
            ),
        };

    Ok(VaultConfigRecord {
        version: match version {
            VaultVersion::Legacy => VaultPuzzleVersion::Legacy,
            VaultVersion::ForceSingletonRecreation => VaultPuzzleVersion::ForceSingletonRecreation,
        },
        source: match config.source {
            VaultConfigSource::Memo => ApiVaultConfigSource::Memo,
            VaultConfigSource::Reconstructed => ApiVaultConfigSource::Reconstructed,
            VaultConfigSource::Unknown => ApiVaultConfigSource::Unknown,
        },
        custody: signers_record(custody, signers)?,
        recovery: recovery
            .map(|recovery| signers_record(recovery, signers))
            .transpose()?,
        recovery_timelock,
        recovery_custody_hash,
    })
}

fn signers_record(
    vault_signers: &VaultSigners,
    signers: &[(u32, PublicKey)],
) -> Result<VaultSignersRecord> {
    Ok(VaultSignersRecord {
        threshold: vault_signers.threshold.try_into().unwrap_or(u32::MAX),
        members: vault_signers
            .members
            .iter()
            .map(|member| {
                let (kind, key, fast_forward) = match member {
                    VaultMember::Bls {
                        public_key,
                        fast_forward,
                    } => (
                        VaultMemberKind::Bls,
                        hex::encode(public_key.to_bytes()),
                        *fast_forward,
                    ),
                    VaultMember::Secp256r1 {
                        public_key,
                        fast_forward,
                    } => (
                        VaultMemberKind::Secp256r1,
                        hex::encode(public_key.to_bytes()),
                        *fast_forward,
                    ),
                    VaultMember::Secp256k1 {
                        public_key,
                        fast_forward,
                    } => (
                        VaultMemberKind::Secp256k1,
                        hex::encode(public_key.to_bytes()),
                        *fast_forward,
                    ),
                    VaultMember::Passkey {
                        public_key,
                        fast_forward,
                    } => (
                        VaultMemberKind::Passkey,
                        hex::encode(public_key.to_bytes()),
                        *fast_forward,
                    ),
                    VaultMember::Singleton {
                        launcher_id,
                        fast_forward,
                    } => (
                        VaultMemberKind::Vault,
                        encode_vault_id(*launcher_id)?,
                        *fast_forward,
                    ),
                    VaultMember::Custom { puzzle_hash } => {
                        (VaultMemberKind::Custom, hex::encode(puzzle_hash), false)
                    }
                };

                let signer_fingerprint = match member {
                    VaultMember::Bls { public_key, .. } => signers
                        .iter()
                        .find(|(_, signer)| signer == public_key)
                        .map(|(fingerprint, _)| *fingerprint),
                    _ => None,
                };

                Ok(VaultMemberRecord {
                    kind,
                    key,
                    fast_forward,
                    signer_fingerprint,
                })
            })
            .collect::<Result<_>>()?,
    })
}
