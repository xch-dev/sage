use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use chia_wallet_sdk::prelude::*;
use sage_api::OptionAssets;
use sage_api::{Amount, NftRoyalty, OfferAsset, OfferSummary};
use sage_database::OfferStatus;
use sage_wallet::WalletError;

use crate::utils::offer_status::offer_expiration;
use crate::ConfirmationInfo;
use crate::StatusCoinType;
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
        let offered_royalties = offer.offered_royalty_amounts();
        let requested_royalties = offer.requested_royalty_amounts();

        let mut maker = Vec::new();

        if offered_amounts.xch > 0 || offered_royalties.xch > 0 {
            let Some(asset) = wallet.db.asset(Bytes32::default()).await? else {
                return Err(Error::Wallet(WalletError::MissingAsset(Bytes32::default())));
            };

            maker.push(OfferAsset {
                amount: Amount::u64(offered_amounts.xch),
                royalty: Amount::u64(offered_royalties.xch),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: None,
            });
        }

        for (asset_id, amount) in offered_amounts.cats {
            let hidden_puzzle_hash = offer
                .asset_info()
                .cat(asset_id)
                .and_then(|cat| cat.hidden_puzzle_hash);

            maker.push(OfferAsset {
                amount: Amount::u64(amount),
                royalty: Amount::u64(offered_royalties.cats.get(&asset_id).copied().unwrap_or(0)),
                asset: self.encode_asset(self.cache_cat(asset_id, hidden_puzzle_hash).await?)?,
                nft_royalty: None,
                option_assets: None,
            });
        }

        for (&launcher_id, nft) in &offer.offered_coins().nfts {
            let asset = self
                .cache_nft(
                    &ctx,
                    launcher_id,
                    nft.info.metadata.ptr(),
                    &mut ConfirmationInfo::default(),
                )
                .await?;

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
            });
        }

        let mut taker = Vec::new();

        if requested_amounts.xch > 0 || requested_royalties.xch > 0 {
            let Some(asset) = wallet.db.asset(Bytes32::default()).await? else {
                return Err(Error::Wallet(WalletError::MissingAsset(Bytes32::default())));
            };

            taker.push(OfferAsset {
                amount: Amount::u64(requested_amounts.xch),
                royalty: Amount::u64(requested_royalties.xch),
                asset: self.encode_asset(asset)?,
                nft_royalty: None,
                option_assets: None,
            });
        }

        for (asset_id, amount) in requested_amounts.cats {
            let hidden_puzzle_hash = offer
                .asset_info()
                .cat(asset_id)
                .and_then(|cat| cat.hidden_puzzle_hash);

            taker.push(OfferAsset {
                amount: Amount::u64(amount),
                royalty: Amount::u64(*requested_royalties.cats.get(&asset_id).unwrap_or(&0)),
                asset: self.encode_asset(self.cache_cat(asset_id, hidden_puzzle_hash).await?)?,
                nft_royalty: None,
                option_assets: None,
            });
        }

        for (&launcher_id, payments) in &offer.requested_payments().nfts {
            let amount = payments
                .iter()
                .map(|p| p.payments.iter().map(|p| p.amount).sum::<u64>())
                .sum::<u64>();

            let nft = offer
                .asset_info()
                .nft(launcher_id)
                .ok_or(DriverError::MissingAssetInfo)?;

            let asset = self
                .cache_nft(
                    &ctx,
                    launcher_id,
                    nft.metadata.ptr(),
                    &mut ConfirmationInfo::default(),
                )
                .await?;

            taker.push(OfferAsset {
                amount: Amount::u64(amount),
                royalty: Amount::u64(0),
                asset: self.encode_asset(asset)?,
                nft_royalty: Some(NftRoyalty {
                    royalty_address: Address::new(nft.royalty_puzzle_hash, self.network().prefix())
                        .encode()?,
                    royalty_basis_points: nft.royalty_basis_points,
                }),
                option_assets: None,
            });
        }

        for (&launcher_id, payments) in &offer.requested_payments().options {
            let amount = payments
                .iter()
                .map(|p| p.payments.iter().map(|p| p.amount).sum::<u64>())
                .sum::<u64>();

            let asset = self.cache_option(launcher_id).await?;

            let Some(row) = wallet.db.option_assets(launcher_id).await? else {
                return Err(Error::MissingOption(launcher_id));
            };

            taker.push(OfferAsset {
                amount: Amount::u64(amount),
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

        let peer_state = self.peer_state.lock().await;

        if let Some(expiration_height) = summary.expiration_height {
            if let Some((height, _)) = peer_state.peak() {
                if height >= expiration_height {
                    offer_status = OfferStatus::Expired;
                }
            }
        }

        if !status.coins.is_empty() {
            if let Some(peer) = peer_state.acquire_peer() {
                let coin_states = peer
                    .fetch_coins(
                        status.coins.keys().copied().collect(),
                        wallet.genesis_challenge,
                    )
                    .await?;

                for coin_state in coin_states {
                    let Some(coin_type) = status.coins.get(&coin_state.coin.coin_id()) else {
                        continue;
                    };

                    match coin_type {
                        StatusCoinType::Cancel { fast_forwardable } => {
                            if coin_state.spent_height.is_some() && !*fast_forwardable {
                                offer_status = OfferStatus::Cancelled;
                            }
                        }
                        StatusCoinType::Settle => {
                            offer_status = OfferStatus::Completed;
                            break;
                        }
                    }
                }
            }
        }

        Ok((summary, offer_status))
    }
}
