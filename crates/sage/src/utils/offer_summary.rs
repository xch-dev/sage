use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use chia_wallet_sdk::prelude::*;
use itertools::Itertools;
use sage_api::OptionAssets;
use sage_api::{Amount, NftRoyalty, OfferAsset, OfferSummary};
use sage_database::{NftUpdaterKindExt, OfferStatus};
use sage_wallet::{
    FETCH_COINS_TIMEOUT, MAX_PEER_ATTEMPTS, OfferSettlement, WalletError, fetch_coins_from_peers,
};

use crate::ConfirmationInfo;
use crate::StatusCoinType;
use crate::encode_nft_updater_kind;
use crate::utils::offer_status::offer_expiration;
use crate::{Error, Result, Sage};

impl Sage {
    pub(crate) async fn summarize_offer(
        &self,
        spend_bundle: SpendBundle,
    ) -> Result<(OfferSummary, OfferStatus)> {
        let wallet = self.wallet()?;

        let mut ctx = SpendContext::new();

        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;

        // Get expiration information
        let status = offer_expiration(&mut ctx, &offer)?;

        let offered_amounts = offer.offered_coins().amounts();
        let requested_amounts = offer.requested_payments().amounts();
        let offered_royalties = offer.offered_royalty_amounts()?;
        let requested_royalties = offer.requested_royalty_amounts()?;

        let mut maker = Vec::new();

        if offered_amounts.xch > 0 || offered_royalties.xch > 0 {
            let Some(asset) = wallet.db.asset(Bytes32::default()).await? else {
                return Err(Error::Wallet(WalletError::MissingAsset(Bytes32::default())));
            };

            maker.push(OfferAsset {
                amount: Amount::u128(offered_amounts.xch),
                royalty: Amount::u128(offered_royalties.xch),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: None,
                nft_metadata_updater_kind: None,
            });
        }

        for (asset_id, amount) in offered_amounts.cats {
            let hidden_puzzle_hash = offer
                .asset_info()
                .cat(asset_id)
                .and_then(|cat| cat.hidden_puzzle_hash);

            maker.push(OfferAsset {
                amount: Amount::u128(amount),
                royalty: Amount::u128(offered_royalties.cats.get(&asset_id).copied().unwrap_or(0)),
                asset: self.encode_asset(self.cache_cat(asset_id, hidden_puzzle_hash).await?)?,
                nft_royalty: None,
                option_assets: None,
                nft_metadata_updater_kind: None,
            });
        }

        for (&launcher_id, nft) in &offer.offered_coins().nfts {
            let asset = self
                .cache_nft(
                    &ctx,
                    launcher_id,
                    nft.info.metadata.ptr(),
                    nft.info.metadata_updater_puzzle_hash,
                    &mut ConfirmationInfo::default(),
                )
                .await?;

            let kind = nft.info.metadata_updater_kind();

            maker.push(OfferAsset {
                amount: Amount::u64(nft.coin.amount),
                royalty: Amount::u64(0),
                asset: self.encode_asset(asset)?,
                nft_royalty: Some(NftRoyalty {
                    royalty_address: Address::new(
                        nft.info.royalty_puzzle_hash,
                        self.network().prefix(),
                    )
                    .encode()?,
                    royalty_basis_points: nft.info.royalty_basis_points,
                }),
                option_assets: None,
                nft_metadata_updater_kind: Some(encode_nft_updater_kind(kind)),
            });
        }

        for (&launcher_id, option) in &offer.offered_coins().options {
            let asset = self.cache_option(launcher_id).await?;

            let Some(row) = wallet.db.option_assets(launcher_id).await? else {
                return Err(Error::MissingOption(launcher_id));
            };

            maker.push(OfferAsset {
                amount: Amount::u64(option.coin.amount),
                royalty: Amount::u64(0),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: Some(OptionAssets {
                    underlying_asset: self.encode_asset(row.underlying_asset)?,
                    underlying_amount: Amount::u64(row.underlying_amount),
                    strike_asset: self.encode_asset(row.strike_asset)?,
                    strike_amount: Amount::u64(row.strike_amount),
                    expiration_seconds: row.expiration_seconds,
                }),
                nft_metadata_updater_kind: None,
            });
        }

        let mut taker = Vec::new();

        if requested_amounts.xch > 0 || requested_royalties.xch > 0 {
            let Some(asset) = wallet.db.asset(Bytes32::default()).await? else {
                return Err(Error::Wallet(WalletError::MissingAsset(Bytes32::default())));
            };

            taker.push(OfferAsset {
                amount: Amount::u128(requested_amounts.xch),
                royalty: Amount::u128(requested_royalties.xch),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: None,
                nft_metadata_updater_kind: None,
            });
        }

        for (asset_id, amount) in requested_amounts.cats {
            let hidden_puzzle_hash = offer
                .asset_info()
                .cat(asset_id)
                .and_then(|cat| cat.hidden_puzzle_hash);

            taker.push(OfferAsset {
                amount: Amount::u128(amount),
                royalty: Amount::u128(*requested_royalties.cats.get(&asset_id).unwrap_or(&0)),
                asset: self.encode_asset(self.cache_cat(asset_id, hidden_puzzle_hash).await?)?,
                nft_royalty: None,
                option_assets: None,
                nft_metadata_updater_kind: None,
            });
        }

        for (&launcher_id, payments) in &offer.requested_payments().nfts {
            let amount = payments
                .iter()
                .map(|p| {
                    p.payments
                        .iter()
                        .map(|p| u128::from(p.amount))
                        .sum::<u128>()
                })
                .sum::<u128>();

            let nft = offer
                .asset_info()
                .nft(launcher_id)
                .ok_or(DriverError::MissingAssetInfo)?;

            let asset = self
                .cache_nft(
                    &ctx,
                    launcher_id,
                    nft.metadata.ptr(),
                    nft.metadata_updater_puzzle_hash,
                    &mut ConfirmationInfo::default(),
                )
                .await?;

            let kind = nft.metadata_updater_kind();

            taker.push(OfferAsset {
                amount: Amount::u128(amount),
                royalty: Amount::u64(0),
                asset: self.encode_asset(asset)?,
                nft_royalty: Some(NftRoyalty {
                    royalty_address: Address::new(nft.royalty_puzzle_hash, self.network().prefix())
                        .encode()?,
                    royalty_basis_points: nft.royalty_basis_points,
                }),
                option_assets: None,
                nft_metadata_updater_kind: Some(encode_nft_updater_kind(kind)),
            });
        }

        for (&launcher_id, payments) in &offer.requested_payments().options {
            let amount = payments
                .iter()
                .map(|p| {
                    p.payments
                        .iter()
                        .map(|p| u128::from(p.amount))
                        .sum::<u128>()
                })
                .sum::<u128>();

            let asset = self.cache_option(launcher_id).await?;

            let Some(row) = wallet.db.option_assets(launcher_id).await? else {
                return Err(Error::MissingOption(launcher_id));
            };

            taker.push(OfferAsset {
                amount: Amount::u128(amount),
                royalty: Amount::u64(0),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: Some(OptionAssets {
                    underlying_asset: self.encode_asset(row.underlying_asset)?,
                    underlying_amount: Amount::u64(row.underlying_amount),
                    strike_asset: self.encode_asset(row.strike_asset)?,
                    strike_amount: Amount::u64(row.strike_amount),
                    expiration_seconds: row.expiration_seconds,
                }),
                nft_metadata_updater_kind: None,
            });
        }

        let summary = OfferSummary {
            fee: Amount::u64(offer.offered_coins().fee),
            maker,
            taker,
            expiration_height: status.expiration_height,
            expiration_timestamp: status.expiration_timestamp,
        };

        let mut offer_status = OfferStatus::Active;

        if status.expiration_timestamp.is_some_and(|ts| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                >= ts
        }) {
            offer_status = OfferStatus::Expired;
        }

        // Don't hold the peer lock across network requests, which would stall syncing.
        let (peak, peers) = {
            let peer_state = self.peer_state.lock().await;
            let peers = peer_state
                .peers_by_peak()
                .take(MAX_PEER_ATTEMPTS)
                .cloned()
                .collect_vec();
            (peer_state.peak(), peers)
        };

        if let Some(expiration_height) = summary.expiration_height
            && let Some((height, _)) = peak
            && height >= expiration_height
        {
            offer_status = OfferStatus::Expired;
        }

        if !status.coins.is_empty() && !peers.is_empty() {
            let coin_states = fetch_coins_from_peers(
                &peers,
                status.coins.keys().copied().collect(),
                wallet.genesis_challenge,
                FETCH_COINS_TIMEOUT,
            )
            .await?;

            let coin_states = coin_states
                .into_iter()
                .map(|coin_state| (coin_state.coin.coin_id(), coin_state))
                .collect::<HashMap<_, _>>();

            for (coin_id, coin_state) in &coin_states {
                if let Some(StatusCoinType::Cancel { fast_forwardable }) = status.coins.get(coin_id)
                    && coin_state.spent_height.is_some()
                    && !*fast_forwardable
                {
                    offer_status = OfferStatus::Cancelled;
                }
            }

            // Settlement coins can also be created by another offer which spends the same coins,
            // so this checks that they were created by taking this one.
            if OfferSettlement::new(&offer)
                .is_settled_on_peers(&peers, &coin_states)
                .await?
            {
                offer_status = OfferStatus::Completed;
            }
        }

        Ok((summary, offer_status))
    }
}
