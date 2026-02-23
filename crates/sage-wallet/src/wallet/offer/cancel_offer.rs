use chia_wallet_sdk::prelude::*;

use crate::{Wallet, WalletError};

impl Wallet {
    pub async fn cancel_offer(
        &self,
        spend_bundle: SpendBundle,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;

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

        self.spend(&mut ctx, vec![coin_id], &[Action::fee(fee)])
            .await?;

        Ok(ctx.take())
    }
}
