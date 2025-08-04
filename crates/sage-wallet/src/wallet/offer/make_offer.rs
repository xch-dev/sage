use chia::{
    bls::Signature,
    protocol::{Bytes32, SpendBundle},
    puzzles::{
        offer::{NotarizedPayment, Payment},
        Memos,
    },
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::driver::{
    calculate_royalty_payments, calculate_trade_price_amounts, calculate_trade_prices, Action,
    AssetInfo, Id, NftAssetInfo, Offer, OfferAmounts, OptionAssetInfo, RequestedPayments,
    RoyaltyInfo, SpendContext, Spends, TransferNftById,
};
use indexmap::IndexMap;
use itertools::Itertools;
use sage_database::{NftOfferInfo, OptionOfferInfo};

use crate::{Wallet, WalletError};

#[derive(Debug, Default, Clone)]
pub struct Offered {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: Vec<Bytes32>,
    pub options: Vec<Bytes32>,
    pub fee: u64,
    pub p2_puzzle_hash: Option<Bytes32>,
}

#[derive(Debug, Default, Clone)]
pub struct Requested {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: IndexMap<Bytes32, NftOfferInfo>,
    pub options: IndexMap<Bytes32, OptionOfferInfo>,
}

impl Wallet {
    pub async fn make_offer(
        &self,
        offered: Offered,
        requested: Requested,
        expires_at: Option<u64>,
    ) -> Result<SpendBundle, WalletError> {
        let mut ctx = SpendContext::new();

        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let offer_amounts = OfferAmounts {
            xch: offered.xch,
            cats: offered.cats.clone(),
        };

        let requested_amounts = OfferAmounts {
            xch: requested.xch,
            cats: requested.cats.clone(),
        };

        let offer_royalties = requested
            .nfts
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
        let mut actions = vec![Action::fee(offered.fee)];

        if offered.xch > 0 {
            actions.push(Action::send(
                Id::Xch,
                SETTLEMENT_PAYMENT_HASH.into(),
                offered.xch,
                Memos::None,
            ));
        }

        for (asset_id, amount) in offered.cats {
            actions.push(Action::send(
                Id::Existing(asset_id),
                SETTLEMENT_PAYMENT_HASH.into(),
                amount,
                Memos::None,
            ));
        }

        for launcher_id in offered.nfts {
            actions.push(Action::send(
                Id::Existing(launcher_id),
                SETTLEMENT_PAYMENT_HASH.into(),
                1,
                Memos::None,
            ));
        }

        for launcher_id in offered.options {
            actions.push(Action::send(
                Id::Existing(launcher_id),
                SETTLEMENT_PAYMENT_HASH.into(),
                1,
                Memos::None,
            ));
        }

        // Pay royalties
        let royalty_payments =
            calculate_royalty_payments(&mut ctx, &offer_trade_price_amounts, &offer_royalties)?;
        actions.extend(royalty_payments.actions());

        // Pay requested payments
        let p2_puzzle_hash = offered.p2_puzzle_hash.unwrap_or(change_puzzle_hash);
        let hint = ctx.hint(p2_puzzle_hash)?;

        // Add requested payments
        let mut spends = Spends::new(change_puzzle_hash);
        self.select_spends(&mut ctx, &mut spends, vec![], &actions)
            .await?;

        let nonce = Offer::nonce(spends.non_settlement_coin_ids());

        let mut asset_info = AssetInfo::new();
        let mut requested_payments = RequestedPayments::new();

        if requested.xch > 0 {
            requested_payments.xch.push(NotarizedPayment::new(
                nonce,
                vec![Payment::new(p2_puzzle_hash, requested.xch, hint)],
            ));
        }

        for (asset_id, amount) in requested.cats {
            requested_payments
                .cats
                .entry(asset_id)
                .or_default()
                .push(NotarizedPayment::new(
                    nonce,
                    vec![Payment::new(p2_puzzle_hash, amount, hint)],
                ));
        }

        for (launcher_id, nft) in requested.nfts {
            let metadata = ctx.alloc_hashed(&nft.metadata)?;

            requested_payments
                .nfts
                .entry(launcher_id)
                .or_default()
                .push(NotarizedPayment::new(
                    nonce,
                    vec![Payment::new(p2_puzzle_hash, 1, hint)],
                ));

            asset_info.insert_nft(
                launcher_id,
                NftAssetInfo::new(
                    metadata,
                    nft.metadata_updater_puzzle_hash,
                    nft.royalty_puzzle_hash,
                    nft.royalty_basis_points,
                ),
            )?;
        }

        for (launcher_id, option) in requested.options {
            requested_payments
                .options
                .entry(launcher_id)
                .or_default()
                .push(NotarizedPayment::new(
                    nonce,
                    vec![Payment::new(p2_puzzle_hash, 1, hint)],
                ));

            asset_info.insert_option(
                launcher_id,
                OptionAssetInfo::new(
                    option.underlying_coin_hash,
                    option.underlying_delegated_puzzle_hash,
                ),
            )?;
        }

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
            &asset_info,
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

        // Add requested payment assertions
        spends.conditions.required = spends
            .conditions
            .required
            .extend(requested_payments.assertions(&mut ctx, &asset_info)?);

        if let Some(expires_at) = expires_at {
            spends.conditions.required = spends
                .conditions
                .required
                .assert_before_seconds_absolute(expires_at);
        }

        // Finish the spend
        let deltas = spends.apply(&mut ctx, &actions)?;

        self.complete_spends(&mut ctx, &deltas, spends).await?;

        let coin_spends = ctx.take();

        let offer = Offer::from_input_spend_bundle(
            &mut ctx,
            SpendBundle::new(coin_spends, Signature::default()),
            requested_payments,
            asset_info,
        )?;

        Ok(offer.to_spend_bundle(&mut ctx)?)
    }
}
