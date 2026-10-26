use chia_wallet_sdk::prelude::*;
use sage_wallet::{SyncCommand, Transaction, insert_transaction, is_fully_signed};

use crate::{Error, Result, Sage};

impl Sage {
    pub(crate) async fn sign(
        &self,
        coin_spends: Vec<CoinSpend>,
        partial: bool,
    ) -> Result<SpendBundle> {
        let wallet = self.wallet()?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let spend_bundle = wallet
            .sign_transaction(
                SpendBundle::new(coin_spends, Signature::default()),
                &AggSigConstants::new(self.network().agg_sig_me()),
                master_sk,
                partial,
            )
            .await?;

        Ok(spend_bundle)
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
