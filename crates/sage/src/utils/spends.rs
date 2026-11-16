use chia_wallet_sdk::prelude::*;
use sage_wallet::{
    SyncCommand, Transaction, Wallet, WalletError, insert_transaction, is_fully_signed,
    sign_with_secret_keys,
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

        if let Some(summary) = wallet.parse_vault_spend(&spend_bundle.coin_spends).await?
            && summary.changes_custody()
        {
            return Err(WalletError::VaultCustodyChange.into());
        }

        let secret_keys = self.vault_signer_secret_keys(wallet.fingerprint)?;

        if secret_keys.is_empty() && !partial {
            return Err(Error::NoVaultSigners);
        }

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
