use chia_wallet_sdk::{driver::TransferNftById, prelude::*};
use itertools::Itertools;

use crate::{Wallet, WalletError};

#[derive(Debug)]
pub struct TakenOffer {
    pub offer: Offer,
    pub spend_bundle: SpendBundle,
}

impl Wallet {
    pub async fn take_offer(
        &self,
        spend_bundle: SpendBundle,
        fee: u64,
    ) -> Result<TakenOffer, WalletError> {
        let mut ctx = SpendContext::new();
        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;

        let change_puzzle_hash = self.change_p2_puzzle_hash().await?;

        // Make payments
        let mut actions = vec![Action::fee(fee)];

        // Pay the royalty each offered NFT asserts for the trade prices its own maker committed
        // to, including NFTs that are also requested and only pass through settlement. Nothing is
        // pooled across makers, and NFTs that committed to no trade price are owed nothing.
        actions.extend(offer.requested_royalty_payments(&mut ctx)?.actions());

        // Pay requested payments
        let mut spends = Spends::new(change_puzzle_hash);
        spends.add(offer.offered_coins().clone());
        actions.extend(offer.requested_payments().actions());

        // Add requested payments
        let offer_input_coin_ids = offer
            .spend_bundle()
            .coin_spends
            .iter()
            .map(|coin_spend| coin_spend.coin.coin_id())
            .collect_vec();
        self.select_spends_excluding(&mut ctx, &mut spends, &actions, &offer_input_coin_ids)
            .await?;

        // Reset DIDs, and have each requested NFT reveal the trade prices that match the royalties
        // its buyer already paid. Like the offered side, this comes from what the maker committed
        // to (their gross offered amounts), never from the net arbitrage.
        for nft in spends.nfts.values().rev() {
            let nft = nft.last()?;

            if !nft.kind.is_conditions() {
                continue;
            }

            let launcher_id = nft.asset.info.launcher_id;

            actions.insert(
                0,
                Action::update_nft(
                    Id::Existing(launcher_id),
                    vec![],
                    Some(TransferNftById::new(
                        None,
                        offer.requested_nft_trade_prices(launcher_id),
                    )),
                ),
            );
        }

        // Finish the spend
        let deltas = spends.apply(&mut ctx, &actions)?;

        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(TakenOffer {
            offer,
            spend_bundle: SpendBundle::new(ctx.take(), Signature::default()),
        })
    }
}
