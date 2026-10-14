use chia_wallet_sdk::{
    driver::{
        TransferNftById, calculate_royalty_payments, calculate_trade_price_amounts,
        calculate_trade_prices,
    },
    prelude::*,
};
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

        let arbitrage = offer.arbitrage();

        let change_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let requested_amounts = OfferAmounts {
            xch: arbitrage.requested.xch,
            cats: arbitrage.requested.cats.clone(),
        };

        // Make payments
        let mut actions = vec![Action::fee(fee)];

        // Pay royalties on every offered NFT, including ones that are also requested and only
        // pass through settlement. Makers commit to trade prices based on their gross requested
        // amounts, not the net arbitrage.
        let offer_royalties = offer.requested_royalties();
        let offer_trade_price_amounts = calculate_trade_price_amounts(
            &offer.requested_payments().amounts(),
            offer_royalties.len(),
        );
        let royalty_payments =
            calculate_royalty_payments(&mut ctx, &offer_trade_price_amounts, &offer_royalties)?;
        actions.extend(royalty_payments.actions());

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

        // Reset DIDs and reveal trade prices
        let mut royalty_nft_count = 0;

        for nft in spends.nfts.values().rev() {
            let nft = nft.last()?;

            if !nft.kind.is_conditions() {
                continue;
            }

            if nft.asset.info.royalty_basis_points > 0 {
                royalty_nft_count += 1;
            }
        }

        let trade_prices = calculate_trade_prices(
            &calculate_trade_price_amounts(&requested_amounts, royalty_nft_count),
            offer.asset_info(),
        );

        for nft in spends.nfts.values().rev() {
            let nft = nft.last()?;

            if !nft.kind.is_conditions() {
                continue;
            }

            actions.insert(
                0,
                Action::update_nft(
                    Id::Existing(nft.asset.info.launcher_id),
                    vec![],
                    Some(TransferNftById::new(
                        None,
                        if nft.asset.info.royalty_basis_points > 0 {
                            trade_prices.clone()
                        } else {
                            vec![]
                        },
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
