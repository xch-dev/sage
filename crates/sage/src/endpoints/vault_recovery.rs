use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sage_api::{
    CancelVaultRecovery, CancelVaultRecoveryResponse, FinishVaultRecovery,
    FinishVaultRecoveryResponse, RekeyVault, RekeyVaultResponse, StartVaultRecovery,
    StartVaultRecoveryResponse, VaultKeySource, VaultRecoveryFinishBlocker, VaultRecoveryRecord,
};
use sage_config::WalletKind as ConfigWalletKind;
use sage_database::{Database, VaultConfig, VaultMember, VaultSigners};
use sage_wallet::{
    DEFAULT_RECOVERY_TIMELOCK, MIN_RECOVERY_TIMELOCK, PendingRecovery, Wallet, WalletError,
    WalletKind, sign_with_secret_keys,
};
use tracing::{info, warn};

use crate::{
    ConfirmationInfo, Error, ParsedKey, Result, Sage, json_spend, parse_amount, parse_key,
};

use super::{vault_mint::SignerKey, vaults::signers_record};

/// How many blocks back from the peak are checked for a transaction block, whose timestamp is the
/// time on chain.
const MAX_TIMESTAMP_BLOCKS: u32 = 32;

impl Sage {
    pub async fn start_vault_recovery(
        &mut self,
        req: StartVaultRecovery,
    ) -> Result<StartVaultRecoveryResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let (_, config) = wallet.active_vault().await?;
        let recovery = config
            .recovery
            .clone()
            .ok_or(WalletError::VaultHasNoRecovery)?;

        // The recovery key is an attached signer, or a mnemonic that's attached once the recovery
        // is submitted.
        let entered = match &req.recovery_mnemonic {
            Some(mnemonic) => {
                let signer = mnemonic_signer(mnemonic)?;

                if !is_bls_member(&recovery.signers, signer.public_key()) {
                    return Err(Error::NotVaultMember(signer.public_key().get_fingerprint()));
                }

                Some(signer)
            }
            None => None,
        };

        let mut recovery_keys = self.member_secret_keys(wallet.fingerprint, &recovery.signers)?;
        recovery_keys.extend(entered.iter().map(|signer| signer.secret_key.clone()));

        if recovery_keys.len() < recovery.signers.threshold {
            return Err(Error::VaultKeysMissing("recovery"));
        }

        let custody = self.vault_key(&req.custody)?;

        if is_bls_member(&recovery.signers, custody.public_key()) {
            return Err(Error::VaultKeysNotDistinct);
        }

        let post_recovery = VaultConfig::with_recovery(
            VaultSigners::new(vec![VaultMember::bls(custody.public_key())], 1),
            recovery.signers.clone(),
            recovery.timelock,
            VaultVersion::ForceSingletonRecreation,
        );

        let recovery_public_keys: Vec<PublicKey> =
            recovery_keys.iter().map(SecretKey::public_key).collect();

        let start = wallet
            .start_vault_recovery(&post_recovery, &recovery_public_keys)
            .await?;

        let mut coin_spends = start.coin_spends.clone();
        let fee_wallet = self
            .pay_fee_alongside(&mut coin_spends, fee, req.fee_fingerprint, start.vault_coin)
            .await?;

        let summary = self
            .summarize(coin_spends.clone(), ConfirmationInfo::default())
            .await?;

        let response = StartVaultRecoveryResponse {
            custody_fingerprint: custody.public_key().get_fingerprint(),
            recovery_timelock: recovery.timelock,
            summary,
            coin_spends: coin_spends.iter().map(json_spend).collect(),
        };

        if !req.auto_submit {
            return Ok(response);
        }

        let spend_bundle = self
            .sign_vault_transaction(coin_spends, &recovery_keys, fee_wallet.as_ref())
            .await?;

        // The new custody key is attached first, so that the vault can be spent by it once the
        // recovery finishes, even if Sage closes right after.
        let mut attach = vec![custody.public_key().get_fingerprint()];
        self.store_signer(&custody)?;

        if let Some(entered) = &entered {
            self.store_signer(entered)?;
            attach.push(entered.public_key().get_fingerprint());
        }

        self.attach_vault_signers(wallet.fingerprint, &attach)?;
        wallet.record_started_recovery(start.recovery).await?;

        self.submit(spend_bundle).await?;

        info!(
            "Submitted a recovery of vault wallet {} to custody key {}",
            wallet.fingerprint, response.custody_fingerprint
        );

        Ok(response)
    }

    pub async fn cancel_vault_recovery(
        &mut self,
        req: CancelVaultRecovery,
    ) -> Result<CancelVaultRecoveryResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let recovery = wallet.pending_recovery().await?;
        let custody_keys = self.member_secret_keys(wallet.fingerprint, &recovery.custody)?;

        if custody_keys.len() < recovery.custody.threshold {
            return Err(Error::VaultKeysMissing("custody"));
        }

        let coin_spends = wallet.cancel_vault_recovery(fee).await?;

        self.finish_vault_transaction(coin_spends, req.auto_submit, &custody_keys)
            .await
    }

    pub async fn finish_vault_recovery(
        &mut self,
        req: FinishVaultRecovery,
    ) -> Result<FinishVaultRecoveryResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let recovery = wallet.pending_recovery().await?;

        let Some((_, post_recovery)) = &recovery.finish else {
            return Err(WalletError::VaultRecoveryUnfinishable.into());
        };

        // Finishing needs no signature, so anyone can finish a recovery. Sage only finishes one that
        // moves the vault to a custody it holds a key for, so that it can't be used to hand the
        // vault to whoever started it.
        let held = self.held_member_keys(&post_recovery.custody)?;

        if held.is_empty() {
            return Err(Error::VaultRecoveryNotOurs);
        }

        let chain_timestamp = self.chain_timestamp(&wallet).await?.unwrap_or(0);
        let finish = wallet.finish_vault_recovery(chain_timestamp).await?;

        let mut coin_spends = finish.coin_spends;
        let fee_wallet = self
            .pay_fee_alongside(
                &mut coin_spends,
                fee,
                req.fee_fingerprint,
                finish.vault_coin,
            )
            .await?;

        let summary = self
            .summarize(coin_spends.clone(), ConfirmationInfo::default())
            .await?;

        let response = FinishVaultRecoveryResponse {
            summary,
            coin_spends: coin_spends.iter().map(json_spend).collect(),
        };

        if !req.auto_submit {
            return Ok(response);
        }

        let spend_bundle = self
            .sign_vault_transaction(coin_spends, &[], fee_wallet.as_ref())
            .await?;

        // The vault is spent by the new custody once the recovery finishes.
        self.attach_vault_signers(wallet.fingerprint, &held)?;

        self.submit(spend_bundle).await?;

        Ok(response)
    }

    pub async fn rekey_vault(&mut self, req: RekeyVault) -> Result<RekeyVaultResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let (_, config) = wallet.active_vault().await?;

        let custody_keys = self.member_secret_keys(wallet.fingerprint, &config.custody)?;

        if custody_keys.len() < config.custody.threshold {
            return Err(Error::VaultKeysMissing("custody"));
        }

        if let Some(timelock) = req.recovery_timelock
            && timelock < MIN_RECOVERY_TIMELOCK
        {
            return Err(Error::VaultRecoveryTimelockTooShort(MIN_RECOVERY_TIMELOCK));
        }

        let new_custody = req
            .custody
            .as_ref()
            .map(|key| self.vault_key(key))
            .transpose()?;
        let new_recovery = req
            .recovery
            .as_ref()
            .map(|key| self.vault_key(key))
            .transpose()?;

        let custody = match &new_custody {
            Some(key) => VaultSigners::new(vec![VaultMember::bls(key.public_key())], 1),
            None => config.custody.clone(),
        };

        let recovery = match (&new_recovery, &config.recovery) {
            (Some(key), _) => Some(VaultSigners::new(
                vec![VaultMember::bls(key.public_key())],
                1,
            )),
            (None, Some(recovery)) => Some(recovery.signers.clone()),
            (None, None) => None,
        };

        let new_config = match recovery {
            Some(recovery) => {
                if custody
                    .members
                    .iter()
                    .any(|member| recovery.members.contains(member))
                {
                    return Err(Error::VaultKeysNotDistinct);
                }

                let timelock = req
                    .recovery_timelock
                    .or(config.recovery.as_ref().map(|recovery| recovery.timelock))
                    .unwrap_or(DEFAULT_RECOVERY_TIMELOCK);

                VaultConfig::with_recovery(
                    custody,
                    recovery,
                    timelock,
                    VaultVersion::ForceSingletonRecreation,
                )
            }
            None => VaultConfig::without_recovery(custody),
        };

        let coin_spends = wallet.rekey_vault(&new_config, fee).await?;

        let summary = self
            .summarize(coin_spends.clone(), ConfirmationInfo::default())
            .await?;

        let response = RekeyVaultResponse {
            custody_fingerprint: new_custody
                .as_ref()
                .map(|key| key.public_key().get_fingerprint()),
            recovery_fingerprint: new_recovery
                .as_ref()
                .map(|key| key.public_key().get_fingerprint()),
            summary,
            coin_spends: coin_spends.iter().map(json_spend).collect(),
        };

        if !req.auto_submit {
            return Ok(response);
        }

        let spend_bundle = self
            .sign_vault_transaction(coin_spends, &custody_keys, None)
            .await?;

        let mut attach = Vec::new();

        for key in new_custody.iter().chain(new_recovery.iter()) {
            self.store_signer(key)?;
            attach.push(key.public_key().get_fingerprint());
        }

        self.attach_vault_signers(wallet.fingerprint, &attach)?;

        self.submit(spend_bundle).await?;

        Ok(response)
    }

    /// The pending recovery of a vault wallet, as shown in its record.
    pub(crate) async fn vault_recovery_record(
        &self,
        wallet: &Wallet,
        chain_timestamp: Option<u64>,
        signers: &[(u32, PublicKey)],
    ) -> Result<Option<VaultRecoveryRecord>> {
        let recovery = match wallet.vault_recovery().await {
            Ok(recovery) => recovery,
            Err(
                WalletError::VaultNoRecoveryPending
                | WalletError::VaultNotSynced
                | WalletError::VaultMelted,
            ) => return Ok(None),
            Err(error) => {
                warn!("Couldn't check the vault's pending recovery: {error}");
                return Ok(None);
            }
        };

        let can_cancel = self
            .member_secret_keys(wallet.fingerprint, &recovery.custody)
            .is_ok_and(|keys| keys.len() >= recovery.custody.threshold);

        Ok(Some(VaultRecoveryRecord {
            started_locally: recovery.is_started_locally,
            can_cancel,
            finish_blocker: self.finish_blocker(&recovery, chain_timestamp)?,
            post_recovery: recovery
                .finish
                .as_ref()
                .map(|(_, post_recovery)| signers_record(&post_recovery.custody, signers))
                .transpose()?,
        }))
    }

    fn finish_blocker(
        &self,
        recovery: &PendingRecovery,
        chain_timestamp: Option<u64>,
    ) -> Result<Option<VaultRecoveryFinishBlocker>> {
        let Some((_, post_recovery)) = &recovery.finish else {
            return Ok(Some(VaultRecoveryFinishBlocker::UnknownCustody));
        };

        if self.held_member_keys(&post_recovery.custody)?.is_empty() {
            return Ok(Some(VaultRecoveryFinishBlocker::NotOurs));
        }

        if !chain_timestamp.is_some_and(|timestamp| recovery.can_finish_at(timestamp)) {
            return Ok(Some(VaultRecoveryFinishBlocker::Timelock));
        }

        Ok(None)
    }

    /// The latest time on chain, from the latest synced block timestamp, or the peak's if a peer is
    /// connected. The local clock isn't used, since it can run ahead of the chain.
    pub(crate) async fn chain_timestamp(&self, wallet: &Wallet) -> Result<Option<u64>> {
        let mut latest = wallet.db.latest_timestamp().await?;

        // Block timestamps aren't synced in tests, as the peer simulator doesn't serve block
        // headers, so they're only stored.
        if self.test {
            return Ok(latest);
        }

        let (peer, peak) = {
            let state = self.peer_state.lock().await;
            (state.acquire_peer(), state.peak())
        };

        if let (Some(peer), Some((height, _))) = (peer, peak) {
            for height in (height.saturating_sub(MAX_TIMESTAMP_BLOCKS)..=height).rev() {
                match peer.block_timestamp_if_transaction_block(height).await {
                    Ok(Some(timestamp)) => {
                        latest = latest.max(Some(timestamp));
                        break;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        warn!("Couldn't fetch the timestamp of block {height}: {error}");
                        break;
                    }
                }
            }
        }

        Ok(latest)
    }

    /// The secret keys of a vault wallet's attached signers which are BLS members of the signers.
    fn member_secret_keys(
        &self,
        fingerprint: u32,
        signers: &VaultSigners,
    ) -> Result<Vec<SecretKey>> {
        Ok(self
            .vault_signer_secret_keys(fingerprint)?
            .into_iter()
            .filter(|secret_key| is_bls_member(signers, secret_key.public_key()))
            .collect())
    }

    /// The fingerprints of the keychain keys with secret keys that are BLS members of the signers.
    fn held_member_keys(&self, signers: &VaultSigners) -> Result<Vec<u32>> {
        let mut held = Vec::new();

        for member in &signers.members {
            let VaultMember::Bls {
                public_key,
                fast_forward: false,
            } = member
            else {
                continue;
            };

            let fingerprint = public_key.get_fingerprint();

            if !self.keychain.contains(fingerprint) {
                continue;
            }

            if let (_, Some(secret_key)) = self.keychain.extract_secrets(fingerprint, b"")?
                && secret_key.public_key() == *public_key
            {
                held.push(fingerprint);
            }
        }

        Ok(held)
    }

    /// A new vault key.
    fn vault_key(&self, source: &VaultKeySource) -> Result<SignerKey> {
        match source {
            VaultKeySource::Keychain { fingerprint } => self.keychain_signer(*fingerprint),
            VaultKeySource::Mnemonic { mnemonic } => mnemonic_signer(mnemonic),
            VaultKeySource::Derived { fingerprint } => {
                if !matches!(self.wallet_kind(*fingerprint)?, WalletKind::Bls { .. }) {
                    return Err(WalletError::VaultFeeWalletNotBls.into());
                }

                let master_sk = self.keychain_signer(*fingerprint)?.secret_key;

                Ok(SignerKey {
                    secret_key: self.unused_custody_key(&master_sk)?,
                    mnemonic: None,
                })
            }
        }
    }

    /// Attaches keychain keys to a vault wallet as its signers.
    fn attach_vault_signers(&mut self, fingerprint: u32, keys: &[u32]) -> Result<()> {
        if let Some(ConfigWalletKind::Vault { signers, .. }) = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|wallet| wallet.fingerprint == fingerprint)
            .map(|wallet| &mut wallet.kind)
        {
            for &key in keys {
                if !signers.contains(&key) {
                    signers.push(key);
                }
            }
        }

        self.save_keychain()?;
        self.save_config()?;

        if let Some(wallet) = self
            .wallet
            .as_ref()
            .filter(|wallet| wallet.fingerprint == fingerprint)
        {
            wallet.set_vault_signer_keys(self.vault_signer_public_keys(fingerprint)?);
        }

        Ok(())
    }

    /// Adds the spends of a BLS wallet that pays a fee for a vault spend which can't pay its own,
    /// such as a recovery. Returns the wallet and its key, which signs for them.
    async fn pay_fee_alongside(
        &self,
        coin_spends: &mut Vec<CoinSpend>,
        fee: u64,
        fee_fingerprint: Option<u32>,
        vault_coin: Coin,
    ) -> Result<Option<(Wallet, SecretKey)>> {
        if fee == 0 {
            return Ok(None);
        }

        let fingerprint = fee_fingerprint.ok_or(Error::VaultFeeWalletRequired)?;

        let kind = self.wallet_kind(fingerprint)?;

        if !matches!(kind, WalletKind::Bls { .. }) {
            return Err(WalletError::VaultFeeWalletNotBls.into());
        }

        let wallet = Wallet::new(
            Database::new(self.connect_to_database(fingerprint).await?),
            self.connect_to_shared_database(fingerprint).await?,
            fingerprint,
            kind,
            self.network().genesis_challenge,
            AggSigConstants::new(self.network().agg_sig_me()),
            None,
        );
        wallet.set_transaction_expiration(self.wallet_config.defaults.transaction_expiration());

        let master_sk = self.master_secret_key(&wallet)?;

        let fee_spends = wallet.pay_fee_alongside(fee, vault_coin.coin_id()).await?;

        // The fee wallet isn't synced while the vault wallet is open, so its coins are checked
        // against the chain, rather than letting the transaction fail later.
        let peer = self
            .peer_state
            .lock()
            .await
            .acquire_peer()
            .ok_or(Error::NoPeers)?;

        for coin_spend in &fee_spends {
            let state = peer
                .fetch_optional_coin(coin_spend.coin.coin_id(), self.network().genesis_challenge)
                .await?;

            if state.is_none_or(|state| state.spent_height.is_some()) {
                return Err(WalletError::MissingCoin(coin_spend.coin.coin_id()).into());
            }
        }

        coin_spends.extend(fee_spends);

        Ok(Some((wallet, master_sk)))
    }

    /// Signs a vault transaction with the vault's keys, and the BLS wallet that pays its fee.
    ///
    /// Unlike [`Sage::sign`], this doesn't refuse spends that change the vault's custody, since
    /// recovering and rekeying a vault are meant to.
    async fn sign_vault_transaction(
        &self,
        coin_spends: Vec<CoinSpend>,
        vault_keys: &[SecretKey],
        fee_wallet: Option<&(Wallet, SecretKey)>,
    ) -> Result<SpendBundle> {
        let agg_sig_constants = AggSigConstants::new(self.network().agg_sig_me());

        let mut spend_bundle = sign_with_secret_keys(
            SpendBundle::new(coin_spends, Signature::default()),
            &agg_sig_constants,
            vault_keys,
            true,
        )?;

        if let Some((wallet, master_sk)) = fee_wallet {
            spend_bundle = wallet
                .sign_transaction(spend_bundle, &agg_sig_constants, master_sk.clone(), true)
                .await?;
        }

        if !self.is_fully_signed(
            &spend_bundle.coin_spends,
            &spend_bundle.aggregated_signature,
        )? {
            return Err(Error::NoSigningKey);
        }

        Ok(spend_bundle)
    }

    /// Summarizes a vault transaction, and signs and submits it with the keys if `auto_submit`.
    async fn finish_vault_transaction(
        &self,
        coin_spends: Vec<CoinSpend>,
        auto_submit: bool,
        vault_keys: &[SecretKey],
    ) -> Result<sage_api::TransactionResponse> {
        let summary = self
            .summarize(coin_spends.clone(), ConfirmationInfo::default())
            .await?;

        let response = sage_api::TransactionResponse {
            summary,
            coin_spends: coin_spends.iter().map(json_spend).collect(),
        };

        if auto_submit {
            let spend_bundle = self
                .sign_vault_transaction(coin_spends, vault_keys, None)
                .await?;
            self.submit(spend_bundle).await?;
        }

        Ok(response)
    }
}

/// A key from a mnemonic, which must have its secret key.
fn mnemonic_signer(mnemonic: &str) -> Result<SignerKey> {
    let ParsedKey {
        master_sk,
        mnemonic,
        ..
    } = parse_key(mnemonic.trim())?;

    let (Some(secret_key), Some(mnemonic)) = (master_sk, mnemonic) else {
        return Err(Error::VaultRecoveryKeySource);
    };

    Ok(SignerKey {
        secret_key,
        mnemonic: Some(mnemonic),
    })
}

fn is_bls_member(signers: &VaultSigners, public_key: PublicKey) -> bool {
    signers.members.iter().any(|member| {
        matches!(member, VaultMember::Bls { public_key: member, fast_forward: false } if *member == public_key)
    })
}
