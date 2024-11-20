use chia::protocol::{CoinSpend, SpendBundle};
use chia_wallet_sdk::AggSigConstants;
use sage_wallet::{insert_transaction, SyncCommand, Transaction};

use crate::{Error, Result, Sage};

use super::parse_genesis_challenge;

impl Sage {
    pub(crate) async fn sign(&self, coin_spends: Vec<CoinSpend>) -> Result<SpendBundle> {
        let wallet = self.wallet()?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let spend_bundle = wallet
            .sign_transaction(
                coin_spends,
                &AggSigConstants::new(parse_genesis_challenge(self.network().agg_sig_me.clone())?),
                master_sk,
            )
            .await?;

        Ok(spend_bundle)
    }

    pub(crate) async fn submit(&self, spend_bundle: SpendBundle) -> Result<()> {
        let wallet = self.wallet()?;

        let mut tx = wallet.db.tx().await?;

        let subscriptions = insert_transaction(
            &mut tx,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        tx.commit().await?;

        self.command_sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await?;

        Ok(())
    }
}