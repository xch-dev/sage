use chia_wallet_sdk::prelude::*;
use sage_wallet::{
    SyncCommand, Transaction, Wallet, WalletError, insert_transaction, is_fully_signed,
    is_placeholder_spend, sign_with_secret_keys,
};

use crate::{Error, Result, Sage};

impl Sage {
    pub(crate) async fn sign(
        &self,
        coin_spends: Vec<CoinSpend>,
        partial: bool,
    ) -> Result<SpendBundle> {
        let wallet = self.wallet()?;

        self.sign_with(
            &wallet,
            SpendBundle::new(coin_spends, Signature::default()),
            partial,
        )
        .await
    }

    /// Signs a spend bundle with the wallet's keys: a BLS wallet's derived keys, or the keys of a
    /// vault's signers.
    ///
    /// A vault's custody signs over its whole transaction, so what the vault spend does is checked
    /// first with clear signing. A spend that changes the vault's custody or melts it is refused,
    /// since nothing Sage makes through this path does either.
    pub(crate) async fn sign_with(
        &self,
        wallet: &Wallet,
        spend_bundle: SpendBundle,
        partial: bool,
    ) -> Result<SpendBundle> {
        let agg_sig_constants = AggSigConstants::new(self.network().agg_sig_me());

        if wallet.vault_launcher_id().is_none() {
            let master_sk = self.master_secret_key(wallet)?;

            return Ok(wallet
                .sign_transaction(spend_bundle, &agg_sig_constants, master_sk, partial)
                .await?);
        }

        let summary = wallet.parse_vault_spend(&spend_bundle.coin_spends).await?;

        if let Some(summary) = &summary
            && summary.changes_custody()
        {
            return Err(WalletError::VaultCustodyChange.into());
        }

        let secret_keys = self.vault_signer_secret_keys(wallet.fingerprint)?;

        if secret_keys.is_empty() && !partial {
            return Err(Error::NoVaultSigners);
        }

        check_vault_signatures(
            &spend_bundle.coin_spends,
            summary.map(|summary| summary.coin.coin_id()),
            &secret_keys
                .iter()
                .map(SecretKey::public_key)
                .collect::<Vec<_>>(),
            &agg_sig_constants,
        )?;

        Ok(sign_with_secret_keys(
            spend_bundle,
            &agg_sig_constants,
            &secret_keys,
            partial,
        )?)
    }

    pub(crate) fn is_fully_signed(
        &self,
        coin_spends: &[CoinSpend],
        signature: &Signature,
    ) -> Result<bool> {
        Ok(is_fully_signed(
            coin_spends,
            signature,
            &AggSigConstants::new(self.network().agg_sig_me()),
        )?)
    }

    /// Submits a transaction. If it was created by Sage with handle recipients, they're
    /// resolved again first, and it's only submitted if they still resolve to the same
    /// addresses.
    pub(crate) async fn submit(&self, spend_bundle: SpendBundle) -> Result<()> {
        let handle_sends = self.recheck_handle_sends(&spend_bundle.coin_spends).await?;

        if handle_sends.is_empty() {
            return self.submit_unchecked(spend_bundle).await;
        }

        let coin_spends = spend_bundle.coin_spends.clone();

        if let Err(error) = self.submit_unchecked(spend_bundle).await {
            // It may be submitted again, and has to be checked again then.
            self.remember_handle_sends(&coin_spends, handle_sends);
            return Err(error);
        }

        self.record_handle_sends(&handle_sends).await;
        Ok(())
    }

    pub(crate) async fn submit_unchecked(&self, spend_bundle: SpendBundle) -> Result<()> {
        let wallet = self.wallet()?;
        let peer = self
            .peer_state
            .lock()
            .await
            .acquire_peer()
            .ok_or(Error::NoPeers)?;

        let subscriptions = insert_transaction(
            &wallet.db,
            &peer,
            wallet.genesis_challenge,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        self.command_sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await?;

        Ok(())
    }
}

/// Checks that a vault's keys are only asked to sign for the vault's own custody spend, which clear
/// signing parsed as `vault_coin_id`.
///
/// A vault's signers only ever sign `AGG_SIG_ME` for the vault coin, over the delegated puzzle that
/// clear signing checked. A spend from a dApp could otherwise ask them to sign for another coin, or
/// include a second spend of the vault coin with a different delegated puzzle than the one that was
/// checked, and the signature would authorize that one instead.
fn check_vault_signatures(
    coin_spends: &[CoinSpend],
    vault_coin_id: Option<Bytes32>,
    signer_keys: &[PublicKey],
    agg_sig_constants: &AggSigConstants,
) -> Result<()> {
    let mut allocator = Allocator::new();
    let mut vault_spends = 0;

    for coin_spend in coin_spends {
        if is_placeholder_spend(coin_spend) {
            continue;
        }

        let coin_id = coin_spend.coin.coin_id();

        if Some(coin_id) == vault_coin_id {
            vault_spends += 1;
        }

        let required =
            RequiredSignature::from_coin_spend(&mut allocator, coin_spend, agg_sig_constants)
                .map_err(WalletError::from)?;

        for required in required {
            let RequiredSignature::Bls(required) = required else {
                continue;
            };

            if !signer_keys.contains(&required.public_key) {
                continue;
            }

            let is_vault_spend = Some(coin_id) == vault_coin_id
                && required.domain_string == Some(agg_sig_constants.me())
                && required.appended_info == coin_id.to_vec();

            if !is_vault_spend {
                return Err(Error::VaultSignatureNotAllowed);
            }
        }
    }

    if vault_spends > 1 {
        return Err(Error::VaultSignatureNotAllowed);
    }

    Ok(())
}
