use chia::protocol::{CoinSpend, SpendBundle};
use chia_wallet_sdk::driver::{Action, Offer, SpendContext};
use sage_database::CoinKind;

use crate::{Wallet, WalletError};

impl Wallet {
    pub async fn cancel_offer(
        &self,
        spend_bundle: SpendBundle,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;

        let mut coin_states = Vec::new();

        for coin_spend in &offer.spend_bundle().coin_spends {
            let Some(row) = self.db.full_coin_state(coin_spend.coin.coin_id()).await? else {
                continue;
            };

            if row.coin_state.created_height.is_none() {
                continue;
            }

            if row.coin_state.spent_height.is_some() {
                return Err(WalletError::UncancellableOffer);
            }

            match row.kind {
                CoinKind::Xch | CoinKind::Cat | CoinKind::Did | CoinKind::Nft => {}
                CoinKind::Unknown => continue,
            }

            coin_states.push(row);
        }

        coin_states.sort_by_key(|row| row.kind);

        let Some(row) = coin_states.first().copied() else {
            return Err(WalletError::UncancellableOffer);
        };

        self.spend(
            &mut ctx,
            vec![row.coin_state.coin.coin_id()],
            &[Action::fee(fee)],
        )
        .await?;

        Ok(ctx.take())
    }
}
