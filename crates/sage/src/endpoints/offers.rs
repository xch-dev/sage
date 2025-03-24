use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use chia::{protocol::SpendBundle, puzzles::nft::NftMetadata};
use chia_wallet_sdk::{
    driver::{Offer, SpendContext},
    signer::AggSigConstants,
    utils::Address,
};
use chrono::{Local, TimeZone};
use clvmr::Allocator;
use indexmap::IndexMap;
use sage_api::{
    Amount, CancelOffer, CancelOfferResponse, CatAmount, CombineOffers, CombineOffersResponse,
    DeleteOffer, DeleteOfferResponse, GetOffer, GetOfferResponse, GetOffers, GetOffersResponse,
    ImportOffer, ImportOfferResponse, MakeOffer, MakeOfferResponse, OfferAssets, OfferCat,
    OfferNft, OfferRecord, OfferRecordStatus, OfferSummary, OfferXch, TakeOffer, TakeOfferResponse,
    ViewOffer, ViewOfferResponse,
};
use sage_assets::fetch_uris_with_hash;
use sage_database::{OfferCatRow, OfferNftRow, OfferRow, OfferStatus, OfferXchRow};
use sage_wallet::{
    aggregate_offers, calculate_royalties, fetch_nft_offer_details, insert_transaction,
    parse_locked_coins, parse_offer_payments, sort_offer, MakerSide, NftRoyaltyInfo, SyncCommand,
    TakerSide, Transaction, Wallet,
};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::{
    extract_nft_data, json_bundle, lookup_coin_creation, offer_expiration, parse_amount,
    parse_asset_id, parse_nft_id, parse_offer_id, ConfirmationInfo, Error, ExtractedNftData,
    Result, Sage,
};

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
                MakerSide {
                    xch: offered_xch,
                    cats: offered_cats,
                    nfts: offered_nfts,
                    fee,
                    p2_puzzle_hash,
                },
                TakerSide {
                    xch: requested_xch,
                    cats: requested_cats,
                    nfts: requested_nfts,
                },
                req.expires_at_second,
                false,
                true,
            )
            .await?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let offer = wallet
            .sign_make_offer(
                unsigned,
                &AggSigConstants::new(self.network().agg_sig_me()),
                master_sk,
            )
            .await?;

        let encoded_offer = offer.encode()?;

        if req.auto_import {
            self.import_offer(ImportOffer {
                offer: encoded_offer.clone(),
            })
            .await?;
        }

        Ok(MakeOfferResponse {
            offer: encoded_offer,
            offer_id: hex::encode(SpendBundle::from(offer).name()),
        })
    }

    pub async fn take_offer(&self, req: TakeOffer) -> Result<TakeOfferResponse> {
        let wallet = self.wallet()?;

        let offer = Offer::decode(&req.offer)?;
        let fee = parse_amount(req.fee)?;

        let unsigned = wallet.take_offer(offer, fee, false, true).await?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let spend_bundle = wallet
            .sign_take_offer(
                unsigned,
                &AggSigConstants::new(self.network().agg_sig_me()),
                master_sk,
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
        let offer = self.summarize_offer(Offer::decode(&req.offer)?).await?;

        Ok(ViewOfferResponse { offer })
    }

    pub async fn import_offer(&self, req: ImportOffer) -> Result<ImportOfferResponse> {
        let wallet = self.wallet()?;
        let offer = sort_offer(Offer::decode(&req.offer)?);
        let spend_bundle: SpendBundle = offer.clone().into();
        let offer_id = spend_bundle.name();

        if wallet.db.get_offer(offer_id).await?.is_some() {
            return Ok(ImportOfferResponse {});
        }

        let peer = self.peer_state.lock().await.acquire_peer();

        let mut allocator = Allocator::new();
        let parsed_offer = offer.parse(&mut allocator)?;

        let (maker, coin_ids) = parse_locked_coins(&mut allocator, &parsed_offer)?;

        let status = if let Some(peer) = peer {
            let coin_creation =
                lookup_coin_creation(&peer, coin_ids.clone(), self.network().genesis_challenge)
                    .await?;
            offer_expiration(&mut allocator, &parsed_offer, &coin_creation)?
        } else {
            warn!("No peers available to fetch coin creation information, so skipping for now");
            offer_expiration(&mut allocator, &parsed_offer, &HashMap::new())?
        };

        let maker_amounts = maker.amounts();
        let mut builder = parsed_offer.take();
        let mut ctx = SpendContext::from(allocator);
        let taker = parse_offer_payments(&mut ctx, &mut builder)?;
        let taker_amounts = taker.amounts();

        let maker_royalties = calculate_royalties(
            &maker.amounts(),
            &taker
                .nfts
                .values()
                .map(|(nft, _payments)| NftRoyaltyInfo {
                    launcher_id: nft.launcher_id,
                    royalty_puzzle_hash: nft.royalty_puzzle_hash,
                    royalty_ten_thousandths: nft.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?
        .amounts();

        let taker_royalties = calculate_royalties(
            &taker_amounts,
            &maker
                .nfts
                .values()
                .map(|nft| NftRoyaltyInfo {
                    launcher_id: nft.info.launcher_id,
                    royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                    royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?
        .amounts();

        let mut cat_rows = Vec::new();
        let mut nft_rows = Vec::new();

        for (asset_id, amount) in maker_amounts.cats {
            let info = wallet.db.cat(asset_id).await?;
            let name = info.as_ref().and_then(|info| info.name.clone());
            let ticker = info.as_ref().and_then(|info| info.ticker.clone());
            let icon = info.as_ref().and_then(|info| info.icon.clone());

            cat_rows.push(OfferCatRow {
                offer_id,
                requested: false,
                asset_id,
                amount,
                name: name.clone(),
                ticker: ticker.clone(),
                icon: icon.clone(),
                royalty: maker_royalties.cats.get(&asset_id).copied().unwrap_or(0),
            });
        }

        for nft in maker.nfts.into_values() {
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

            nft_rows.push(OfferNftRow {
                offer_id,
                requested: false,
                launcher_id: nft.info.launcher_id,
                royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                name: info.name,
                thumbnail: info.icon,
                thumbnail_mime_type: Some("image/png".to_string()),
            });
        }

        for (asset_id, amount) in taker_amounts.cats {
            let info = wallet.db.cat(asset_id).await?;
            let name = info.as_ref().and_then(|info| info.name.clone());
            let ticker = info.as_ref().and_then(|info| info.ticker.clone());
            let icon = info.as_ref().and_then(|info| info.icon.clone());

            cat_rows.push(OfferCatRow {
                offer_id,
                requested: true,
                asset_id,
                amount,
                name: name.clone(),
                ticker: ticker.clone(),
                icon: icon.clone(),
                royalty: taker_royalties.cats.get(&asset_id).copied().unwrap_or(0),
            });
        }

        for (nft, _) in taker.nfts.into_values() {
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

            nft_rows.push(OfferNftRow {
                offer_id,
                requested: true,
                launcher_id: nft.launcher_id,
                royalty_puzzle_hash: nft.royalty_puzzle_hash,
                royalty_ten_thousandths: nft.royalty_ten_thousandths,
                name: info.name,
                thumbnail: info.icon,
                thumbnail_mime_type,
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
            fee: maker.fee,
            status: OfferStatus::Active,
            inserted_timestamp,
        })
        .await?;

        for coin_id in coin_ids {
            tx.insert_offered_coin(offer_id, coin_id).await?;
        }

        if maker_amounts.xch > 0 || maker_royalties.xch > 0 {
            tx.insert_offer_xch(OfferXchRow {
                offer_id,
                requested: false,
                amount: maker_amounts.xch,
                royalty: maker_royalties.xch,
            })
            .await?;
        }

        if taker_amounts.xch > 0 || taker_royalties.xch > 0 {
            tx.insert_offer_xch(OfferXchRow {
                offer_id,
                requested: true,
                amount: taker_amounts.xch,
                royalty: taker_royalties.xch,
            })
            .await?;
        }

        for row in cat_rows {
            tx.insert_offer_cat(row).await?;
        }

        for row in nft_rows {
            tx.insert_offer_nft(row).await?;
        }

        tx.commit().await?;

        Ok(ImportOfferResponse {})
    }

    pub fn combine_offers(&self, req: CombineOffers) -> Result<CombineOffersResponse> {
        let offers = req
            .offers
            .iter()
            .map(|offer| Ok(Offer::decode(offer)?))
            .collect::<Result<Vec<_>>>()?;

        Ok(CombineOffersResponse {
            offer: aggregate_offers(offers).encode()?,
        })
    }

    pub async fn get_offers(&self, _req: GetOffers) -> Result<GetOffersResponse> {
        let wallet = self.wallet()?;
        let offers = wallet.db.get_offers().await?;

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
            .get_offer(offer_id)
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
        let xch = wallet.db.offer_xch(offer.offer_id).await?;
        let cats = wallet.db.offer_cats(offer.offer_id).await?;
        let nfts = wallet.db.offer_nfts(offer.offer_id).await?;

        let mut maker_xch_amount = 0u128;
        let mut maker_xch_royalty = 0u128;
        let mut taker_xch_amount = 0u128;
        let mut taker_xch_royalty = 0u128;

        for xch in xch {
            if xch.requested {
                taker_xch_amount += xch.amount as u128;
                taker_xch_royalty += xch.royalty as u128;
            } else {
                maker_xch_amount += xch.amount as u128;
                maker_xch_royalty += xch.royalty as u128;
            }
        }

        let mut maker_cats = IndexMap::new();
        let mut taker_cats = IndexMap::new();

        for cat in cats {
            let asset_id = hex::encode(cat.asset_id);

            let record = OfferCat {
                amount: Amount::u64(cat.amount),
                royalty: Amount::u64(cat.royalty),
                name: cat.name,
                ticker: cat.ticker,
                icon_url: cat.icon,
            };

            if cat.requested {
                taker_cats.insert(asset_id, record);
            } else {
                maker_cats.insert(asset_id, record);
            }
        }

        let mut maker_nfts = IndexMap::new();
        let mut taker_nfts = IndexMap::new();

        for nft in nfts {
            let nft_id = Address::new(nft.launcher_id, "nft".to_string()).encode()?;

            let record = OfferNft {
                royalty_address: Address::new(nft.royalty_puzzle_hash, self.network().prefix())
                    .encode()?,
                royalty_ten_thousandths: nft.royalty_ten_thousandths,
                name: nft.name,
                icon: nft.thumbnail.map(|data| BASE64_STANDARD.encode(data)),
            };

            if nft.requested {
                taker_nfts.insert(nft_id, record);
            } else {
                maker_nfts.insert(nft_id, record);
            }
        }

        Ok(OfferRecord {
            offer_id: hex::encode(offer.offer_id),
            offer: offer.encoded_offer,
            status: match offer.status {
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
                        amount: Amount::u128(maker_xch_amount),
                        royalty: Amount::u128(maker_xch_royalty),
                    },
                    cats: maker_cats,
                    nfts: maker_nfts,
                },
                taker: OfferAssets {
                    xch: OfferXch {
                        amount: Amount::u128(taker_xch_amount),
                        royalty: Amount::u128(taker_xch_royalty),
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

        let Some(row) = wallet.db.get_offer(offer_id).await? else {
            return Err(Error::MissingOffer(offer_id));
        };

        let offer = Offer::decode(&row.encoded_offer)?;
        let coin_spends = wallet.cancel_offer(offer, fee, false, true).await?;

        self.transact(coin_spends, req.auto_submit).await
    }
}
