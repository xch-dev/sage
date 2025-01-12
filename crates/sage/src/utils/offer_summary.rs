use std::time::Duration;
use tracing::debug;

use chia::{clvm_traits::FromClvm, puzzles::nft::NftMetadata};
use chia_wallet_sdk::{encode_address, Offer, SpendContext};
use indexmap::IndexMap;
use sage_api::{Amount, OfferAssets, OfferCat, OfferNft, OfferSummary, OfferXch};
use sage_assets::fetch_uris_with_hash;
use sage_wallet::{calculate_royalties, parse_locked_coins, parse_offer_payments, NftRoyaltyInfo};
use tokio::time::timeout;

use crate::{Result, Sage};

use super::{extract_nft_data, ConfirmationInfo, ExtractedNftData};

impl Sage {
    pub(crate) async fn summarize_offer(&self, offer: Offer) -> Result<OfferSummary> {
        debug!("Starting offer summary");
        let wallet = self.wallet()?;
        debug!("Got wallet");

        let mut ctx = SpendContext::new();
        debug!("Created spend context");

        let offer = offer.parse(&mut ctx.allocator)?;
        debug!("Parsed offer");

        let (locked_coins, _original_coin_ids) = parse_locked_coins(&mut ctx.allocator, &offer)?;
        debug!("Parsed locked coins");

        let maker_amounts = locked_coins.amounts();
        debug!("Got maker amounts");

        let mut builder = offer.take();
        debug!("Got offer builder");

        let requested_payments = parse_offer_payments(&mut ctx, &mut builder)?;
        debug!("Parsed offer payments");

        let taker_amounts = requested_payments.amounts();
        debug!("Got taker amounts");

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
        debug!("Calculated maker royalties");

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
        debug!("Calculated taker royalties");

        let maker_royalties = maker_royalties.amounts();
        let taker_royalties = taker_royalties.amounts();
        debug!("Got royalty amounts");

        let mut maker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(maker_amounts.xch),
                royalty: Amount::u64(maker_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };
        debug!("Created maker assets");

        for (asset_id, amount) in maker_amounts.cats {
            debug!("Processing maker CAT: {}", hex::encode(asset_id));
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
        debug!("Processed maker CATs");

        for (launcher_id, nft) in locked_coins.nfts {
            debug!("Processing maker NFT: {}", hex::encode(launcher_id));
            let info = if let Ok(metadata) =
                NftMetadata::from_clvm(&ctx.allocator, nft.info.metadata.ptr())
            {
                debug!("Parsed NFT metadata");
                let mut confirmation_info = ConfirmationInfo::default();

                if let Some(hash) = metadata.data_hash {
                    debug!("Fetching NFT data hash: {}", hex::encode(hash));
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
                    debug!("Fetching NFT metadata hash: {}", hex::encode(hash));
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
                debug!("Failed to parse NFT metadata, using default");
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
        debug!("Processed maker NFTs");

        let mut taker = OfferAssets {
            xch: OfferXch {
                amount: Amount::u64(taker_amounts.xch),
                royalty: Amount::u64(taker_royalties.xch),
            },
            cats: IndexMap::new(),
            nfts: IndexMap::new(),
        };
        debug!("Created taker assets");

        for (asset_id, amount) in taker_amounts.cats {
            debug!("Processing taker CAT: {}", hex::encode(asset_id));
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
        debug!("Processed taker CATs");

        for (launcher_id, (nft, _payments)) in requested_payments.nfts {
            debug!("Processing taker NFT: {}", hex::encode(launcher_id));
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
        debug!("Processed taker NFTs");

        Ok(OfferSummary {
            fee: Amount::u64(locked_coins.fee),
            maker,
            taker,
        })
    }
}
