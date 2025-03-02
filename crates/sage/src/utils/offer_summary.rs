use std::time::Duration;
use std::collections::HashMap;

use chia::{clvm_traits::FromClvm, puzzles::nft::NftMetadata};
use chia_wallet_sdk::{encode_address, Offer, SpendContext};
use chrono::{Local, TimeZone};
use indexmap::IndexMap;
use sage_api::{Amount, OfferAssets, OfferCat, OfferNft, OfferSummary, OfferXch};
use sage_assets::fetch_uris_with_hash;
use sage_wallet::{calculate_royalties, parse_locked_coins, parse_offer_payments, NftRoyaltyInfo};
use tokio::time::timeout;
use tracing::warn;

use crate::{Result, Sage, parse_genesis_challenge};
use crate::utils::offer_status::{lookup_coin_creation, offer_expiration};

use super::{extract_nft_data, ConfirmationInfo, ExtractedNftData};

impl Sage {
    pub(crate) async fn summarize_offer(&self, offer: Offer) -> Result<OfferSummary> {
        let wallet = self.wallet()?;

        let mut ctx = SpendContext::new();

        let parsed_offer = offer.parse(&mut ctx.allocator)?;
        let (locked_coins, coin_ids) = parse_locked_coins(&mut ctx.allocator, &parsed_offer)?;
        let maker_amounts = locked_coins.amounts();

        // Get expiration information
        let peer = self.peer_state.lock().await.acquire_peer();
        let status = if let Some(peer) = peer {
            let coin_creation = lookup_coin_creation(
                &peer,
                coin_ids.clone(),
                parse_genesis_challenge(self.network().genesis_challenge.clone())?,
            )
            .await?;
            offer_expiration(&mut ctx.allocator, &parsed_offer, &coin_creation)?
        } else {
            warn!("No peers available to fetch coin creation information, so skipping for now");
            offer_expiration(&mut ctx.allocator, &parsed_offer, &HashMap::new())?
        };

        let mut builder = parsed_offer.take();
        let requested_payments = parse_offer_payments(&mut ctx, &mut builder)?;
        let taker_amounts = requested_payments.amounts();

        let maker_royalties = calculate_royalties(
            &maker_amounts,
            &requested_payments
                .nfts
                .values()
                .map(|(nft, _payments)| NftRoyaltyInfo {
                    launcher_id: nft.launcher_id,
                    royalty_puzzle_hash: nft.royalty_puzzle_hash,
                    royalty_ten_thousandths: nft.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?;

        let taker_royalties = calculate_royalties(
            &taker_amounts,
            &locked_coins
                .nfts
                .values()
                .map(|nft| NftRoyaltyInfo {
                    launcher_id: nft.info.launcher_id,
                    royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                    royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?;

        let maker_royalties = maker_royalties.amounts();
        let taker_royalties = taker_royalties.amounts();

        let mut maker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(maker_amounts.xch),
                royalty: Amount::u64(maker_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };

        for (asset_id, amount) in maker_amounts.cats {
            let cat = wallet.db.cat(asset_id).await?;

            maker.cats.insert(
                hex::encode(asset_id),
                OfferCat {
                    amount: Amount::u64(amount),
                    royalty: Amount::u64(maker_royalties.cats.get(&asset_id).copied().unwrap_or(0)),
                    name: cat.as_ref().and_then(|cat| cat.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.icon.clone()),
                },
            );
        }

        for (launcher_id, nft) in locked_coins.nfts {
            let info = if let Ok(metadata) =
                NftMetadata::from_clvm(&ctx.allocator, nft.info.metadata.ptr())
            {
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
                encode_address(launcher_id.to_bytes(), "nft")?,
                OfferNft {
                    image_data: info.image_data,
                    image_mime_type: info.image_mime_type,
                    name: info.name,
                    royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                    royalty_address: encode_address(
                        nft.info.royalty_puzzle_hash.into(),
                        &self.network().address_prefix,
                    )?,
                },
            );
        }

        let mut taker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(taker_amounts.xch),
                royalty: Amount::u64(taker_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };

        for (asset_id, amount) in taker_amounts.cats {
            let cat = wallet.db.cat(asset_id).await?;

            taker.cats.insert(
                hex::encode(asset_id),
                OfferCat {
                    amount: Amount::u64(amount),
                    royalty: Amount::u64(taker_royalties.cats.get(&asset_id).copied().unwrap_or(0)),
                    name: cat.as_ref().and_then(|cat| cat.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.icon.clone()),
                },
            );
        }

        for (launcher_id, (nft, _payments)) in requested_payments.nfts {
            let metadata = NftMetadata::from_clvm(&ctx.allocator, nft.metadata.ptr())?;
            let info = extract_nft_data(
                Some(&wallet.db),
                Some(metadata),
                &ConfirmationInfo::default(),
            )
            .await?;

            taker.nfts.insert(
                encode_address(launcher_id.to_bytes(), "nft")?,
                OfferNft {
                    image_data: info.image_data,
                    image_mime_type: info.image_mime_type,
                    name: info.name,
                    royalty_ten_thousandths: nft.royalty_ten_thousandths,
                    royalty_address: encode_address(
                        nft.royalty_puzzle_hash.into(),
                        &self.network().address_prefix,
                    )?,
                },
            );
        }

        Ok(OfferSummary {
            fee: Amount::u64(locked_coins.fee),
            maker,
            taker,
            expiration_height: status.expiration_height,
            expiration_timestamp: status.expiration_timestamp,
            expiration_date: status.expiration_timestamp.map(|timestamp| {
                Local
                    .timestamp_opt(timestamp.try_into().unwrap_or(0), 0)
                    .unwrap()
                    .format("%b %d, %Y %r")
                    .to_string()
            }),
        })
    }
}
