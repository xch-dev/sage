use std::collections::HashMap;
use std::time::Duration;

use base64::{prelude::BASE64_STANDARD, Engine};
use chia::{protocol::SpendBundle, puzzles::nft::NftMetadata};
use chia_wallet_sdk::driver::{DriverError, Offer};
use chia_wallet_sdk::{driver::SpendContext, utils::Address};
use indexmap::IndexMap;
use itertools::Itertools;
use sage_api::{Amount, OfferAssets, OfferCat, OfferNft, OfferSummary, OfferXch};
use sage_assets::fetch_uris_with_hash;
use tokio::time::timeout;
use tracing::warn;

use crate::utils::offer_status::{lookup_coin_creation, offer_expiration};
use crate::{Result, Sage};

use super::{extract_nft_data, ConfirmationInfo, ExtractedNftData};

impl Sage {
    pub(crate) async fn summarize_offer(&self, spend_bundle: SpendBundle) -> Result<OfferSummary> {
        let wallet = self.wallet()?;

        let mut ctx = SpendContext::new();

        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;
        let coin_ids = offer
            .cancellable_coin_spends()?
            .into_iter()
            .map(|cs| cs.coin.coin_id())
            .collect_vec();

        // Get expiration information
        let peer = self.peer_state.lock().await.acquire_peer();
        let status = if let Some(peer) = peer {
            let coin_creation =
                lookup_coin_creation(&peer, coin_ids.clone(), self.network().genesis_challenge)
                    .await?;
            offer_expiration(&mut ctx, &offer, &coin_creation)?
        } else {
            warn!("No peers available to fetch coin creation information, so skipping for now");
            offer_expiration(&mut ctx, &offer, &HashMap::new())?
        };

        let offered_amounts = offer.offered_coins().amounts();
        let requested_amounts = offer.requested_payments().amounts();
        let offered_royalties = offer.offered_royalty_amounts();
        let requested_royalties = offer.requested_royalty_amounts();

        let mut maker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(offered_amounts.xch),
                royalty: Amount::u64(offered_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };

        for (asset_id, amount) in offered_amounts.cats {
            let cat = wallet.db.cat_asset(asset_id).await?;

            maker.cats.insert(
                hex::encode(asset_id),
                OfferCat {
                    amount: Amount::u64(amount),
                    royalty: Amount::u64(
                        offered_royalties.cats.get(&asset_id).copied().unwrap_or(0),
                    ),
                    name: cat.as_ref().and_then(|cat| cat.asset.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.asset.icon_url.clone()),
                },
            );
        }

        for (&launcher_id, nft) in &offer.offered_coins().nfts {
            let info = if let Ok(metadata) = ctx.extract::<NftMetadata>(nft.info.metadata.ptr()) {
                let mut confirmation_info = ConfirmationInfo::default();

                if let Some(hash) = metadata.data_hash {
                    if let Ok(Some(data)) = timeout(
                        Duration::from_secs(10),
                        fetch_uris_with_hash(metadata.data_uris.clone(), hash),
                    )
                    .await
                    {
                        confirmation_info.nft_data.insert(hash, data);
                    }
                }

                if let Some(hash) = metadata.metadata_hash {
                    if let Ok(Some(data)) = timeout(
                        Duration::from_secs(10),
                        fetch_uris_with_hash(metadata.metadata_uris.clone(), hash),
                    )
                    .await
                    {
                        confirmation_info.nft_data.insert(hash, data);
                    }
                }

                extract_nft_data(Some(&wallet.db), Some(metadata), &confirmation_info).await?
            } else {
                ExtractedNftData::default()
            };

            maker.nfts.insert(
                Address::new(launcher_id, "nft".to_string()).encode()?,
                OfferNft {
                    icon: info.icon.map(|icon| BASE64_STANDARD.encode(icon)),
                    name: info.name,
                    royalty_ten_thousandths: nft.info.royalty_basis_points,
                    royalty_address: Address::new(
                        nft.info.royalty_puzzle_hash,
                        self.network().prefix().clone(),
                    )
                    .encode()?,
                },
            );
        }

        let mut taker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(requested_amounts.xch),
                royalty: Amount::u64(requested_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };

        for (asset_id, amount) in requested_amounts.cats {
            let cat = wallet.db.cat_asset(asset_id).await?;

            taker.cats.insert(
                hex::encode(asset_id),
                OfferCat {
                    amount: Amount::u64(amount),
                    royalty: Amount::u64(
                        requested_royalties
                            .cats
                            .get(&asset_id)
                            .copied()
                            .unwrap_or(0),
                    ),
                    name: cat.as_ref().and_then(|cat| cat.asset.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.asset.icon_url.clone()),
                },
            );
        }

        for &launcher_id in offer.requested_payments().nfts.keys() {
            let nft = offer
                .asset_info()
                .nft(launcher_id)
                .ok_or(DriverError::MissingAssetInfo)?;

            let metadata = ctx.extract::<NftMetadata>(nft.metadata.ptr())?;
            let info = extract_nft_data(
                Some(&wallet.db),
                Some(metadata),
                &ConfirmationInfo::default(),
            )
            .await?;

            taker.nfts.insert(
                Address::new(launcher_id, "nft".to_string()).encode()?,
                OfferNft {
                    icon: info.icon.map(|icon| BASE64_STANDARD.encode(icon)),
                    name: info.name,
                    royalty_ten_thousandths: nft.royalty_basis_points,
                    royalty_address: Address::new(nft.royalty_puzzle_hash, self.network().prefix())
                        .encode()?,
                },
            );
        }

        Ok(OfferSummary {
            fee: Amount::u64(offer.offered_coins().fee),
            maker,
            taker,
            expiration_height: status.expiration_height,
            expiration_timestamp: status.expiration_timestamp,
        })
    }
}
