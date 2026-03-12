use chia_wallet_sdk::{
    driver::{
        TransferNftById, calculate_royalty_payments, calculate_trade_price_amounts,
        calculate_trade_prices,
    },
    prelude::*,
};
use indexmap::IndexMap;
use itertools::Itertools;
use sage_database::NftOfferInfo;

use crate::{Wallet, WalletError};

use super::fee_trade_price_to_nft_trade_prices;

impl Wallet {
    pub async fn take_offer(
        &self,
        spend_bundle: SpendBundle,
        fee: u64,
    ) -> Result<SpendBundle, WalletError> {
        let mut ctx = SpendContext::new();
        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;

        let arbitrage = offer.arbitrage();

        let mut requested_nfts = IndexMap::new();

        for launcher_id in arbitrage.requested.nfts {
            let Some(nft) = offer.asset_info().nft(launcher_id) else {
                return Err(WalletError::MissingNft(launcher_id));
            };

            let metadata = ctx.serialize(&nft.metadata)?;

            requested_nfts.insert(
                launcher_id,
                NftOfferInfo {
                    metadata,
                    metadata_updater_puzzle_hash: nft.metadata_updater_puzzle_hash,
                    royalty_puzzle_hash: nft.royalty_puzzle_hash,
                    royalty_basis_points: nft.royalty_basis_points,
                },
            );
        }

        let change_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let offer_amounts = OfferAmounts {
            xch: arbitrage.offered.xch,
            cats: arbitrage.offered.cats.clone(),
        };

        let requested_amounts = OfferAmounts {
            xch: arbitrage.requested.xch,
            cats: arbitrage.requested.cats.clone(),
        };

        let offer_royalties = requested_nfts
            .iter()
            .map(|(&launcher_id, nft)| {
                RoyaltyInfo::new(
                    launcher_id,
                    nft.royalty_puzzle_hash,
                    nft.royalty_basis_points,
                )
            })
            .filter(|info| info.basis_points > 0)
            .collect_vec();

        let offer_trade_price_amounts =
            calculate_trade_price_amounts(&offer_amounts, offer_royalties.len());

        // Make payments
        let mut actions = vec![Action::fee(fee)];

        // Pay royalties
        let royalty_payments =
            calculate_royalty_payments(&mut ctx, &offer_trade_price_amounts, &offer_royalties)?;
        actions.extend(royalty_payments.actions());

        // Pay requested payments (including transfer fees for fee CATs)
        let mut spends = Spends::new(change_puzzle_hash);
        spends.add(offer.offered_coins().clone());
        actions.extend(offer.take_actions_with_transfer_fees(&mut ctx)?);

        // Add requested payments
        self.select_spends(&mut ctx, &mut spends, &actions).await?;

        // Apply fee CAT trade context to relevant CAT spends
        offer.apply_transfer_fee_trade_context(&mut spends)?;

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

        let fee_trade_prices = calculate_trade_prices(
            &calculate_trade_price_amounts(&requested_amounts, royalty_nft_count),
            offer.asset_info(),
        );
        let nft_trade_prices =
            fee_trade_price_to_nft_trade_prices(&fee_trade_prices, offer.asset_info());

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
                            nft_trade_prices.clone()
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

        Ok(offer.take(SpendBundle::new(ctx.take(), Signature::default())))
    }
}
