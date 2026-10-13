use chia_wallet_sdk::prelude::*;

use crate::{Wallet, WalletError};

impl Wallet {
    pub async fn cancel_offer(
        &self,
        spend_bundle: SpendBundle,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        self.cancel_offers(vec![spend_bundle], fee).await
    }

    /// Cancels every offer in one spend, paying `fee` once for the whole batch.
    ///
    /// Each offer is cancelled by spending one of its own coins back to the wallet. Building
    /// them all in one spend means any extra XCH the fee needs (when an offer holds no XCH of
    /// its own, such as a CAT or NFT offer) is selected once, so two offers can never pick the
    /// same fee coin and produce a bundle that spends it twice.
    pub async fn cancel_offers(
        &self,
        spend_bundles: Vec<SpendBundle>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let mut selected_coin_ids = Vec::with_capacity(spend_bundles.len());

        for spend_bundle in &spend_bundles {
            let offer = Offer::from_spend_bundle(&mut ctx, spend_bundle)?;

            let mut coins = Vec::new();

            for coin_spend in offer.cancellable_coin_spends()? {
                let coin_id = coin_spend.coin.coin_id();

                let Some(kind) = self.db.coin_kind(coin_id).await? else {
                    continue;
                };

                coins.push((kind, coin_id));
            }

            coins.sort();

            let Some((_, coin_id)) = coins.first().copied() else {
                return Err(WalletError::UncancellableOffer);
            };

            if !selected_coin_ids.contains(&coin_id) {
                selected_coin_ids.push(coin_id);
            }
        }

        self.spend(&mut ctx, selected_coin_ids, &[Action::fee(fee)])
            .await?;

        Ok(ctx.take())
    }
}
