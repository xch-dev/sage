use chia::protocol::Bytes32;
use chia::puzzles::nft::NftMetadata;
use chia_wallet_sdk::{
    driver::{decode_offer, encode_offer, DriverError, Offer, SpendContext},
    signer::AggSigConstants,
    utils::Address,
};
use chrono::{Local, TimeZone};
use indexmap::IndexMap;
use itertools::Itertools;
use sage_api::{
    Amount, CancelOffer, CancelOfferResponse, CancelOffers, CancelOffersResponse, CatAmount,
    CombineOffers, CombineOffersResponse, DeleteOffer, DeleteOfferResponse, GetOffer,
    GetOfferResponse, GetOffers, GetOffersResponse, ImportOffer, ImportOfferResponse, MakeOffer,
    MakeOfferResponse, OfferAssets, OfferCat, OfferNft, OfferRecord, OfferRecordStatus,
    OfferSummary, OfferXch, TakeOffer, TakeOfferResponse, ViewOffer, ViewOfferResponse,
};
use sage_assets::fetch_uris_with_hash;
use sage_database::{OfferRow, OfferStatus, OfferedAsset};
use sage_wallet::{
    aggregate_offers, fetch_nft_offer_details, insert_transaction, sort_offer, Offered, Requested,
    SyncCommand, Transaction, Wallet,
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::{
    extract_nft_data, json_bundle, lookup_coin_creation, offer_expiration, parse_amount,
    parse_asset_id, parse_nft_id, parse_offer_id, ConfirmationInfo, Error, ExtractedNftData,
    Result, Sage,
};

#[derive(Debug, Clone)]
struct AssetToOffer {
    offer_id: Bytes32,
    asset_id: Bytes32,
    amount: u64,
    royalty: u64,
    is_requested: bool,
}

impl Sage {
    pub async fn make_offer(&self, req: MakeOffer) -> Result<MakeOfferResponse> {
        let wallet = self.wallet()?;

        let offered_xch = parse_amount(req.offered_assets.xch)?;

        let mut offered_cats = IndexMap::new();

        for CatAmount { asset_id, amount } in req.offered_assets.cats {
            offered_cats.insert(parse_asset_id(asset_id)?, parse_amount(amount)?);
        }

        let mut offered_nfts = Vec::new();

        for nft_id in req.offered_assets.nfts {
            offered_nfts.push(parse_nft_id(nft_id)?);
        }

        let requested_xch = parse_amount(req.requested_assets.xch)?;

        let mut requested_cats = IndexMap::new();

        for CatAmount { asset_id, amount } in req.requested_assets.cats {
            requested_cats.insert(parse_asset_id(asset_id)?, parse_amount(amount)?);
        }

        let mut requested_nfts = IndexMap::new();
        let mut peer = None;

        for nft_id in req.requested_assets.nfts {
            if peer.is_none() {
                peer = self.peer_state.lock().await.acquire_peer();
            }

            let peer = peer.as_ref().ok_or(Error::NoPeers)?;

            let nft_id = parse_nft_id(nft_id)?;

            let Some(offer_details) = fetch_nft_offer_details(peer, nft_id).await? else {
                return Err(Error::CouldNotFetchNft(nft_id));
            };

            requested_nfts.insert(nft_id, offer_details);
        }

        let fee = parse_amount(req.fee)?;

        let p2_puzzle_hash = req
            .receive_address
            .map(|address| self.parse_address(address))
            .transpose()?;

        let unsigned = wallet
            .make_offer(
                Offered {
                    xch: offered_xch,
                    cats: offered_cats,
                    nfts: offered_nfts,
                    fee,
                    p2_puzzle_hash,
                },
                Requested {
                    xch: requested_xch,
                    cats: requested_cats,
                    nfts: requested_nfts,
                },
                req.expires_at_second,
            )
            .await?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let offer = wallet
            .sign_transaction(
                unsigned,
                &AggSigConstants::new(self.network().agg_sig_me()),
                master_sk,
                false,
            )
            .await?;

        let encoded_offer = encode_offer(&offer)?;

        if req.auto_import {
            self.import_offer(ImportOffer {
                offer: encoded_offer.clone(),
            })
            .await?;
        }

        Ok(MakeOfferResponse {
            offer: encoded_offer,
            offer_id: hex::encode(sort_offer(offer).name()),
        })
    }

    pub async fn take_offer(&self, req: TakeOffer) -> Result<TakeOfferResponse> {
        let wallet = self.wallet()?;

        let offer = decode_offer(&req.offer)?;
        let fee = parse_amount(req.fee)?;

        let unsigned = wallet.take_offer(offer, fee).await?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let spend_bundle = wallet
            .sign_transaction(
                unsigned,
                &AggSigConstants::new(self.network().agg_sig_me()),
                master_sk,
                true,
            )
            .await?;

        debug!(
            "{}",
            serde_json::to_string(&json_bundle(&spend_bundle)).expect("msg")
        );

        if req.auto_submit {
            let peer = self
                .peer_state
                .lock()
                .await
                .acquire_peer()
                .ok_or(Error::NoPeers)?;

            let subscriptions = insert_transaction(
                &wallet.db,
                &peer,
                wallet.genesis_challenge,
                spend_bundle.name(),
                Transaction::from_coin_spends(spend_bundle.coin_spends.clone())?,
                spend_bundle.aggregated_signature.clone(),
            )
            .await?;

            self.command_sender
                .send(SyncCommand::SubscribeCoins {
                    coin_ids: subscriptions,
                })
                .await?;
        }

        let json_bundle = json_bundle(&spend_bundle);
        let transaction_id = hex::encode(spend_bundle.name());

        if req.auto_import {
            self.import_offer(ImportOffer { offer: req.offer }).await?;
        }

        Ok(TakeOfferResponse {
            summary: self
                .summarize(spend_bundle.coin_spends, ConfirmationInfo::default())
                .await?,
            spend_bundle: json_bundle,
            transaction_id,
        })
    }

    pub async fn view_offer(&self, req: ViewOffer) -> Result<ViewOfferResponse> {
        let offer = self.summarize_offer(decode_offer(&req.offer)?).await?;

        Ok(ViewOfferResponse { offer })
    }

    pub async fn import_offer(&self, req: ImportOffer) -> Result<ImportOfferResponse> {
        let wallet = self.wallet()?;
        let spend_bundle = sort_offer(decode_offer(&req.offer)?);
        let offer_id = spend_bundle.name();

        if wallet.db.offer(offer_id).await?.is_some() {
            return Ok(ImportOfferResponse {
                offer_id: hex::encode(offer_id),
            });
        }

        let peer = self.peer_state.lock().await.acquire_peer();

        let mut ctx = SpendContext::new();
        let offer = Offer::from_spend_bundle(&mut ctx, &spend_bundle)?;
        let coin_ids = offer
            .cancellable_coin_spends()?
            .into_iter()
            .map(|cs| cs.coin.coin_id())
            .collect_vec();

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

        let mut cat_rows = Vec::new();
        let mut nft_rows = Vec::new();

        for (asset_id, amount) in offered_amounts.cats {
            let info = wallet.db.cat_asset(asset_id).await?;
            let name = info.as_ref().and_then(|info| info.asset.name.clone());
            let ticker = info.as_ref().and_then(|info| info.ticker.clone());
            let icon = info.as_ref().and_then(|info| info.asset.icon_url.clone());

            cat_rows.push(AssetToOffer {
                offer_id,
                is_requested: false,
                asset_id,
                amount,
                royalty: offered_royalties.cats.get(&asset_id).copied().unwrap_or(0),
            });
        }

        for nft in offer.offered_coins().nfts.values() {
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

            nft_rows.push(AssetToOffer {
                offer_id,
                is_requested: false,
                asset_id: nft.info.launcher_id,
                amount: 0, // TODO is this right?
                royalty: nft.info.royalty_basis_points as u64,
            });
        }

        for (asset_id, amount) in requested_amounts.cats {
            let info = wallet.db.cat_asset(asset_id).await?;
            let name = info.as_ref().and_then(|info| info.asset.name.clone());
            let ticker = info.as_ref().and_then(|info| info.ticker.clone());
            let icon = info.as_ref().and_then(|info| info.asset.icon_url.clone());

            cat_rows.push(AssetToOffer {
                offer_id,
                is_requested: true,
                asset_id,
                amount,
                royalty: requested_royalties
                    .cats
                    .get(&asset_id)
                    .copied()
                    .unwrap_or(0),
            });
        }

        for &launcher_id in offer.requested_payments().nfts.keys() {
            let nft = offer
                .asset_info()
                .nft(launcher_id)
                .ok_or(DriverError::MissingAssetInfo)?;

            let info = if let Ok(metadata) = ctx.extract::<NftMetadata>(nft.metadata.ptr()) {
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

            let thumbnail_mime_type = if info.icon.is_some() {
                Some("image/png".to_string())
            } else {
                None
            };

            nft_rows.push(AssetToOffer {
                offer_id,
                is_requested: true,
                asset_id: launcher_id,
                amount: 0, // TODO is this right?
                royalty: nft.royalty_basis_points as u64,
            });
        }

        let mut tx = wallet.db.tx().await?;

        let inserted_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before the UNIX epoch")
            .as_secs();

        tx.insert_offer(OfferRow {
            offer_id,
            encoded_offer: req.offer,
            expiration_height: status.expiration_height,
            expiration_timestamp: status.expiration_timestamp,
            fee: offer.offered_coins().fee,
            status: OfferStatus::Active,
            inserted_timestamp,
        })
        .await?;

        for coin_id in coin_ids {
            tx.insert_offered_coin(offer_id, coin_id).await?;
        }

        if offered_amounts.xch > 0 || offered_royalties.xch > 0 {
            tx.insert_offer_xch(offer_id, offered_amounts.xch, offered_royalties.xch, false)
                .await?;
        }

        if requested_amounts.xch > 0 || requested_royalties.xch > 0 {
            tx.insert_offer_xch(
                offer_id,
                requested_amounts.xch,
                requested_royalties.xch,
                true,
            )
            .await?;
        }

        for row in cat_rows {
            tx.insert_offer_asset(
                row.offer_id,
                row.asset_id,
                row.amount,
                row.royalty,
                row.is_requested,
            )
            .await?;
        }

        for row in nft_rows {
            tx.insert_offer_asset(
                row.offer_id,
                row.asset_id,
                row.amount,
                row.royalty,
                row.is_requested,
            )
            .await?;
        }

        tx.commit().await?;

        Ok(ImportOfferResponse {
            offer_id: hex::encode(offer_id),
        })
    }

    pub fn combine_offers(&self, req: CombineOffers) -> Result<CombineOffersResponse> {
        let offers = req
            .offers
            .iter()
            .map(|offer| Ok(decode_offer(offer)?))
            .collect::<Result<Vec<_>>>()?;

        Ok(CombineOffersResponse {
            offer: encode_offer(&aggregate_offers(offers))?,
        })
    }

    pub async fn get_offers(&self, _req: GetOffers) -> Result<GetOffersResponse> {
        let wallet = self.wallet()?;
        let offers = wallet.db.active_offers().await?;

        let mut records = Vec::new();

        for offer in offers {
            records.push(self.offer_record(&wallet, offer).await?);
        }

        Ok(GetOffersResponse { offers: records })
    }

    pub async fn get_offer(&self, req: GetOffer) -> Result<GetOfferResponse> {
        let wallet = self.wallet()?;

        let offer_id = parse_offer_id(req.offer_id)?;
        let offer = wallet
            .db
            .offer(offer_id)
            .await?
            .ok_or_else(|| Error::MissingOffer(offer_id))?;

        let offer = self.offer_record(&wallet, offer).await?;

        Ok(GetOfferResponse { offer })
    }

    pub async fn delete_offer(&self, req: DeleteOffer) -> Result<DeleteOfferResponse> {
        let wallet = self.wallet()?;
        let offer_id = hex::decode(&req.offer_id)?;

        wallet.db.delete_offer(offer_id.try_into()?).await?;

        Ok(DeleteOfferResponse {})
    }

    async fn offer_record(&self, wallet: &Wallet, offer: OfferRow) -> Result<OfferRecord> {
        let xch = wallet.db.offer_xch_assets(offer.offer_id).await?;
        let cats = wallet.db.offer_cat_assets(offer.offer_id).await?;
        let nfts = wallet.db.offer_nft_assets(offer.offer_id).await?;

        let mut maker_xch_amount = 0;
        let mut maker_xch_royalty = 0;
        let mut taker_xch_amount = 0;
        let mut taker_xch_royalty = 0;

        for xch in xch {
            if xch.is_requested {
                taker_xch_amount += xch.amount;
                taker_xch_royalty += xch.royalty;
            } else {
                maker_xch_amount += xch.amount;
                maker_xch_royalty += xch.royalty;
            }
        }

        let mut maker_cats = IndexMap::new();
        let mut taker_cats = IndexMap::new();

        for cat in cats {
            let asset_id = hex::encode(cat.asset.hash);

            let record = OfferCat {
                amount: Amount::u64(cat.amount),
                royalty: Amount::u64(cat.royalty),
                name: cat.asset.name,
                ticker: Some("TODO".to_string()), // TODO cat.asset.ticker,
                icon_url: cat.asset.icon_url,
            };

            if cat.is_requested {
                taker_cats.insert(asset_id, record);
            } else {
                maker_cats.insert(asset_id, record);
            }
        }

        let mut maker_nfts = IndexMap::new();
        let mut taker_nfts = IndexMap::new();

        for nft in nfts {
            let nft_id = Address::new(nft.asset.hash, "nft".to_string()).encode()?;

            let record = OfferNft {
                royalty_address: "TODO".to_string(), // TODO Address::new(nft.royalty_puzzle_hash, self.network().prefix()).encode()?,
                royalty_ten_thousandths: nft.royalty as u16, // TODO: is this right?
                name: nft.asset.name,
                icon: None, // TODO nft.thumbnail.map(|data| BASE64_STANDARD.encode(data)),
            };

            if nft.is_requested {
                taker_nfts.insert(nft_id, record);
            } else {
                maker_nfts.insert(nft_id, record);
            }
        }

        Ok(OfferRecord {
            offer_id: hex::encode(offer.offer_id),
            offer: offer.encoded_offer,
            status: match offer.status {
                OfferStatus::Pending => OfferRecordStatus::Pending,
                OfferStatus::Active => OfferRecordStatus::Active,
                OfferStatus::Completed => OfferRecordStatus::Completed,
                OfferStatus::Cancelled => OfferRecordStatus::Cancelled,
                OfferStatus::Expired => OfferRecordStatus::Expired,
            },
            creation_date: Local
                .timestamp_opt(offer.inserted_timestamp.try_into()?, 0)
                .unwrap()
                .format("%b %d, %Y %r")
                .to_string(),
            summary: OfferSummary {
                maker: OfferAssets {
                    xch: OfferXch {
                        amount: Amount::u64(maker_xch_amount),
                        royalty: Amount::u64(maker_xch_royalty),
                    },
                    cats: maker_cats,
                    nfts: maker_nfts,
                },
                taker: OfferAssets {
                    xch: OfferXch {
                        amount: Amount::u64(taker_xch_amount),
                        royalty: Amount::u64(taker_xch_royalty),
                    },
                    cats: taker_cats,
                    nfts: taker_nfts,
                },
                fee: Amount::u64(offer.fee),
                expiration_height: offer.expiration_height,
                expiration_timestamp: offer.expiration_timestamp,
            },
        })
    }

    pub async fn cancel_offer(&self, req: CancelOffer) -> Result<CancelOfferResponse> {
        let wallet = self.wallet()?;
        let offer_id = parse_offer_id(req.offer_id)?;
        let fee = parse_amount(req.fee)?;

        let Some(row) = wallet.db.offer(offer_id).await? else {
            return Err(Error::MissingOffer(offer_id));
        };

        let offer = decode_offer(&row.encoded_offer)?;
        let coin_spends = wallet.cancel_offer(offer, fee).await?;

        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn cancel_offers(&self, req: CancelOffers) -> Result<CancelOffersResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let offer_ids = req
            .offer_ids
            .iter()
            .map(|offer_id| parse_offer_id(offer_id.clone()))
            .collect::<Result<Vec<_>>>()?;

        let mut coin_spends = Vec::new();

        for offer_id in offer_ids {
            let Some(row) = wallet.db.offer(offer_id).await? else {
                return Err(Error::MissingOffer(offer_id));
            };

            let offer = decode_offer(&row.encoded_offer)?;
            let spends = wallet.cancel_offer(offer, fee).await?;
            coin_spends.extend(spends);
        }

        self.transact(coin_spends, req.auto_submit).await
    }
}
