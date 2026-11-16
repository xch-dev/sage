use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bip39::Mnemonic;
use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sage_api::{MintVault, MintVaultResponse};
use sage_config::{PendingVaultMint, Wallet as WalletEntry, WalletKind as ConfigWalletKind};
use sage_database::{Database, VaultConfig, VaultMember, VaultSigners};
use sage_wallet::{
    MIN_RECOVERY_TIMELOCK, Status, WalletError, WalletKind, submit_to_peers, track_vault,
    vault_custody_key,
};
use tracing::{info, warn};

use crate::{
    ConfirmationInfo, Error, ParsedKey, Result, Sage, assign_fingerprint, encode_vault_id,
    json_spend, parse_amount, parse_key,
};

/// How many custody keys of a wallet are tried, to find one that no vault wallet uses yet.
const MAX_CUSTODY_KEY_INDEX: u32 = 1000;

/// How long after a mint transaction expires it's still looked for on chain before the mint is
/// considered failed, since block timestamps can lag behind the local clock.
const MINT_EXPIRATION_GRACE: u64 = 10 * 60;

/// A key that signs for a vault, along with its mnemonic if it has one.
pub(crate) struct SignerKey {
    pub(crate) secret_key: SecretKey,
    pub(crate) mnemonic: Option<Mnemonic>,
}

impl SignerKey {
    pub(crate) fn public_key(&self) -> PublicKey {
        self.secret_key.public_key()
    }
}

impl Sage {
    pub async fn mint_vault(&mut self, req: MintVault) -> Result<MintVaultResponse> {
        let wallet = self.wallet()?;

        if !matches!(wallet.kind, WalletKind::Bls { .. }) {
            return Err(WalletError::VaultMintNotSupported.into());
        }

        if req.recovery_timelock < MIN_RECOVERY_TIMELOCK {
            return Err(Error::VaultRecoveryTimelockTooShort(MIN_RECOVERY_TIMELOCK));
        }

        let fee = parse_amount(req.fee)?;

        let custody = match req.custody_fingerprint {
            Some(fingerprint) => self.keychain_signer(fingerprint)?,
            None => SignerKey {
                secret_key: self.unused_custody_key(&self.master_secret_key(&wallet)?)?,
                mnemonic: None,
            },
        };

        let recovery = match (req.recovery_mnemonic, req.recovery_fingerprint) {
            (Some(mnemonic), None) => {
                let ParsedKey {
                    master_sk,
                    mnemonic,
                    ..
                } = parse_key(mnemonic.trim())?;

                let (Some(secret_key), Some(mnemonic)) = (master_sk, mnemonic) else {
                    return Err(Error::VaultRecoveryKeySource);
                };

                SignerKey {
                    secret_key,
                    mnemonic: Some(mnemonic),
                }
            }
            (None, Some(fingerprint)) => self.keychain_signer(fingerprint)?,
            _ => return Err(Error::VaultRecoveryKeySource),
        };

        // The recovery key exists to recover the vault when the custody key is lost, so it must be
        // a different key.
        if recovery.public_key() == custody.public_key() {
            return Err(Error::VaultRecoveryKeySource);
        }

        let config = VaultConfig::with_recovery(
            VaultSigners::new(vec![VaultMember::bls(custody.public_key())], 1),
            VaultSigners::new(vec![VaultMember::bls(recovery.public_key())], 1),
            req.recovery_timelock,
            VaultVersion::ForceSingletonRecreation,
        );

        let mint = wallet.mint_vault(&config, fee).await?;
        let launcher_id = mint.launcher_id;

        let summary = self
            .summarize(mint.coin_spends.clone(), ConfirmationInfo::default())
            .await?;

        let mut response = MintVaultResponse {
            launcher_id: encode_vault_id(launcher_id)?,
            fingerprint: None,
            custody_fingerprint: custody.public_key().get_fingerprint(),
            recovery_fingerprint: recovery.public_key().get_fingerprint(),
            summary,
            coin_spends: mint.coin_spends.iter().map(json_spend).collect(),
        };

        if !req.auto_submit {
            return Ok(response);
        }

        let spend_bundle = self.sign(mint.coin_spends, false).await?;
        let transaction_id = spend_bundle.name();

        let expires_at = match wallet.transaction_expiration() {
            0 => None,
            expiration => Some(now().saturating_add(expiration)),
        };

        // The keys and the wallet are stored before the mint is submitted, so that the vault
        // can't be lost if Sage closes right after. If it can't be submitted, they're removed.
        let added_keys = [
            self.store_signer(&custody)?
                .then_some(response.custody_fingerprint),
            self.store_signer(&recovery)?
                .then_some(response.recovery_fingerprint),
        ];

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
                signers: vec![response.custody_fingerprint, response.recovery_fingerprint],
            },
            vault_mint: Some(PendingVaultMint {
                funding_fingerprint: wallet.fingerprint,
                transaction_id,
                expires_at,
            }),
            ..Default::default()
        });

        let result = async {
            self.save_keychain()?;
            self.save_config()?;

            let db = Database::new(self.connect_to_database(fingerprint).await?);
            let mut tx = db.tx().await?;
            track_vault(&mut tx, launcher_id).await?;
            tx.commit().await?;

            // The vault wallet is usually opened right after, which stops this wallet's sync from
            // submitting the mint later, so it's sent to the peers now. It's also recorded as a
            // pending transaction of this wallet, which resubmits it when it's opened again.
            let peers = self.peer_state.lock().await.peers();

            if let Status::Failed(_status, error) =
                submit_to_peers(&peers, spend_bundle.clone()).await?
            {
                return Err(Error::VaultMintRejected(error.unwrap_or_default()));
            }

            self.submit(spend_bundle).await
        }
        .await;

        if let Err(error) = result {
            self.remove_vault_wallet(fingerprint, &added_keys)?;
            return Err(error);
        }

        info!(
            "Submitted the mint of vault {} as wallet {fingerprint}",
            response.launcher_id
        );

        response.fingerprint = Some(fingerprint);

        Ok(response)
    }

    /// Checks the vault wallets whose mint hasn't been confirmed yet. A mint that's found on chain
    /// is confirmed, and a mint whose transaction can no longer be confirmed is removed, along with
    /// its keys if no other wallet uses them.
    pub(crate) async fn resolve_vault_mints(&mut self) -> Result<()> {
        let pending: Vec<(u32, Bytes32, PendingVaultMint)> = self
            .wallet_config
            .wallets
            .iter()
            .filter_map(|wallet| match (&wallet.kind, wallet.vault_mint) {
                (ConfigWalletKind::Vault { launcher_id, .. }, Some(mint)) => {
                    Some((wallet.fingerprint, *launcher_id, mint))
                }
                _ => None,
            })
            .collect();

        for (fingerprint, launcher_id, mint) in pending {
            match self.vault_mint_status(fingerprint, launcher_id, mint).await {
                Ok(Some(true)) => {
                    if let Some(wallet) = self
                        .wallet_config
                        .wallets
                        .iter_mut()
                        .find(|wallet| wallet.fingerprint == fingerprint)
                    {
                        wallet.vault_mint = None;
                    }

                    self.save_config()?;
                }
                Ok(Some(false)) => {
                    warn!("The mint of vault wallet {fingerprint} failed, so it's removed");

                    let signers = match self.wallet_entry(fingerprint).map(|wallet| &wallet.kind) {
                        Some(ConfigWalletKind::Vault { signers, .. }) => signers.clone(),
                        _ => Vec::new(),
                    };

                    let keys: Vec<Option<u32>> = signers.into_iter().map(Some).collect();
                    self.remove_vault_wallet(fingerprint, &keys)?;

                    if self.config.global.fingerprint.is_none() {
                        self.switch_wallet().await?;
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    warn!("Couldn't check the mint of vault wallet {fingerprint}: {error}");
                }
            }
        }

        Ok(())
    }

    /// Whether a vault mint was confirmed, failed, or is still pending (`None`).
    async fn vault_mint_status(
        &self,
        fingerprint: u32,
        launcher_id: Bytes32,
        mint: PendingVaultMint,
    ) -> Result<Option<bool>> {
        let db = self.vault_database(fingerprint).await?;

        if db
            .vault_status(launcher_id)
            .await?
            .is_some_and(|status| status.coin.is_some())
        {
            return Ok(Some(true));
        }

        let Some(peer) = self.peer_state.lock().await.acquire_peer() else {
            return Ok(None);
        };

        let launcher = peer
            .fetch_optional_coin(launcher_id, self.network().genesis_challenge)
            .await?;

        if launcher.is_some_and(|state| state.created_height.is_some()) {
            return Ok(Some(true));
        }

        let now = now();

        let expired = if let Some(expires_at) = mint.expires_at {
            now > expires_at.saturating_add(MINT_EXPIRATION_GRACE)
        } else {
            // Without an expiration, the mint failed once the funding wallet no longer has the
            // transaction pending, since it would have been found on chain otherwise.
            let funding = self.vault_database(mint.funding_fingerprint).await?;
            !funding
                .mempool_items()
                .await?
                .iter()
                .any(|item| item.hash == mint.transaction_id)
        };

        Ok(expired.then_some(false))
    }

    /// Removes a vault wallet whose mint failed, and the keys that were added for it, unless
    /// another wallet uses them.
    fn remove_vault_wallet(&mut self, fingerprint: u32, added_keys: &[Option<u32>]) -> Result<()> {
        self.wallet_config
            .wallets
            .retain(|wallet| wallet.fingerprint != fingerprint);

        for &key in added_keys.iter().flatten() {
            if !self.is_key_used(key) {
                self.keychain.remove(key);
            }
        }

        if self.config.global.fingerprint == Some(fingerprint) {
            self.config.global.fingerprint = None;
        }

        self.save_keychain()?;
        self.save_config()?;

        let path = self.path.join("wallets").join(fingerprint.to_string());
        if path.try_exists()? {
            fs::remove_dir_all(path)?;
        }

        Ok(())
    }

    /// A keychain key with a secret key, to use as a vault signer.
    pub(crate) fn keychain_signer(&self, fingerprint: u32) -> Result<SignerKey> {
        let (mnemonic, Some(secret_key)) = self.keychain.extract_secrets(fingerprint, b"")? else {
            return Err(Error::NoSigningKey);
        };

        Ok(SignerKey {
            secret_key,
            mnemonic,
        })
    }

    /// Stores a signer's key in the keychain, and returns whether it was added.
    pub(crate) fn store_signer(&mut self, signer: &SignerKey) -> Result<bool> {
        let public_key = signer.public_key();
        let existed = self.keychain.contains(public_key.get_fingerprint());

        self.store_key(
            &public_key,
            Some(&signer.secret_key),
            signer.mnemonic.as_ref(),
            true,
        )?;

        Ok(!existed)
    }

    /// The first custody key derived from the wallet's master key that no vault wallet uses, so
    /// each vault minted from a wallet gets its own key.
    pub(crate) fn unused_custody_key(&self, master_sk: &SecretKey) -> Result<SecretKey> {
        (0..MAX_CUSTODY_KEY_INDEX)
            .map(|index| vault_custody_key(master_sk, index))
            .find(|secret_key| !self.is_key_used(secret_key.public_key().get_fingerprint()))
            .ok_or(Error::NoFreeFingerprint)
    }
}

/// The current time in seconds since the Unix epoch, or 0 if the clock is before it.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
