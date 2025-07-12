use std::time::Duration;

use base64::{prelude::BASE64_STANDARD, Engine};
use chia::protocol::Bytes32;
use chia::{protocol::SpendBundle, puzzles::nft::NftMetadata};
use chia_wallet_sdk::driver::{DriverError, Offer};
use chia_wallet_sdk::{driver::SpendContext, utils::Address};
use sage_api::{Amount, OfferAsset, OfferSummary};
use sage_assets::{base64_data_uri, fetch_uris_with_hash};
use sage_database::Asset;
use sage_wallet::WalletError;
use tokio::time::timeout;

use crate::utils::offer_status::offer_expiration;
use crate::{encode_asset, Error, Result, Sage};

use super::{extract_nft_data, ConfirmationInfo, ExtractedNftData};

impl Sage {
    pub(crate) async fn summarize_offer(&self, spend_bundle: SpendBundle) -> Result<OfferSummary> {
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
                asset: encode_asset(asset)?,
                amount: Amount::u64(offered_amounts.xch),
                royalty: Amount::u64(offered_royalties.xch),
                nft_royalty: None,
            });
        }

        for (asset_id, amount) in offered_amounts.cats {
            let cat = wallet
                .db
                .asset(asset_id)
                .await?
                .unwrap_or_else(|| Asset::default_cat(asset_id));

            maker.push(OfferAsset {
                amount: Amount::u64(amount),
                royalty: Amount::u64(offered_royalties.cats.get(&asset_id).copied().unwrap_or(0)),
                asset: encode_asset(cat)?,
                nft_royalty: None,
            });
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

            maker.push(OfferAsset {
                amount: Amount::u64(nft.coin.amount),
                royalty: Amount::u64(0),
                asset: encode_asset(Asset {
                    hash: launcher_id,
                    name: info.name,
                    ticker: None,
                    precision: 1,
                    icon_url: info.icon.map(|icon| base64_data_uri(&icon, "image/png")),
                    description: None,
                    is_sensitive_content: false,
                    is_visible: true,
                    kind: AssetKind,
                }),
            });

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

        let mut taker = Vec::new();

        if requested_amounts.xch > 0 || requested_royalties.xch > 0 {
            let Some(asset) = wallet.db.asset(Bytes32::default()).await? else {
                return Err(Error::Wallet(WalletError::MissingAsset(Bytes32::default())));
            };

            taker.push(OfferAsset {
                asset: encode_asset(asset)?,
                amount: Amount::u64(requested_amounts.xch),
                royalty: Amount::u64(requested_royalties.xch),
                nft_royalty: None,
            });
        }

        for (asset_id, amount) in requested_amounts.cats {
            let cat = wallet
                .db
                .asset(asset_id)
                .await?
                .unwrap_or_else(|| Asset::default_cat(asset_id));

            let &royalty = requested_royalties.cats.get(&asset_id).unwrap_or(&0);

            taker.push(OfferAsset {
                amount: Amount::u64(amount),
                royalty: Amount::u64(royalty),
                asset: encode_asset(cat)?,
                nft_royalty: None,
            });
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
