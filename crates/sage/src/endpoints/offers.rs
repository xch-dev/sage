use std::time::{SystemTime, UNIX_EPOCH};

use chia::protocol::SpendBundle;
use chia_wallet_sdk::{AggSigConstants, Offer};
use chrono::{Local, TimeZone};
use clvmr::Allocator;
use indexmap::IndexMap;
use sage_api::{
    CatAmount, DeleteOffer, DeleteOfferResponse, GetOffers, GetOffersResponse, ImportOffer,
    ImportOfferResponse, MakeOffer, MakeOfferResponse, OfferRecord, OfferRecordStatus, TakeOffer,
    TakeOfferResponse, ViewOffer, ViewOfferResponse,
};
use sage_database::{OfferRow, OfferStatus};
use sage_wallet::{
    fetch_nft_offer_details, insert_transaction, MakerSide, SyncCommand, TakerSide, Transaction,
};
use tracing::{debug, warn};

use crate::{
    json_bundle, lookup_coin_creation, offer_expiration, parse_asset_id, parse_cat_amount,
    parse_genesis_challenge, parse_nft_id, ConfirmationInfo, Error, OfferExpiration, Result, Sage,
};

impl Sage {
    pub async fn make_offer(&self, req: MakeOffer) -> Result<MakeOfferResponse> {
        let wallet = self.wallet()?;

        let offered_xch = self.parse_amount(req.offered_assets.xch)?;

        let mut offered_cats = IndexMap::new();

        for CatAmount { asset_id, amount } in req.offered_assets.cats {
            offered_cats.insert(parse_asset_id(asset_id)?, parse_cat_amount(amount)?);
        }

        let mut offered_nfts = Vec::new();

        for nft_id in req.offered_assets.nfts {
            offered_nfts.push(parse_nft_id(nft_id)?);
        }

        let requested_xch = self.parse_amount(req.requested_assets.xch)?;

        let mut requested_cats = IndexMap::new();

        for CatAmount { asset_id, amount } in req.requested_assets.cats {
            requested_cats.insert(parse_asset_id(asset_id)?, parse_cat_amount(amount)?);
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

        let fee = self.parse_amount(req.fee)?;

        let unsigned = wallet
            .make_offer(
                MakerSide {
                    xch: offered_xch,
                    cats: offered_cats,
                    nfts: offered_nfts,
                    fee,
                },
                TakerSide {
                    xch: requested_xch,
                    cats: requested_cats,
                    nfts: requested_nfts,
                },
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
                &AggSigConstants::new(parse_genesis_challenge(self.network().agg_sig_me.clone())?),
                master_sk,
            )
            .await?;

        Ok(MakeOfferResponse {
            offer: offer.encode()?,
        })
    }

    pub async fn take_offer(&self, req: TakeOffer) -> Result<TakeOfferResponse> {
        let wallet = self.wallet()?;

        let offer = Offer::decode(&req.offer)?;
        let fee = self.parse_amount(req.fee)?;

        let unsigned = wallet.take_offer(offer, fee, false, true).await?;

        let (_mnemonic, Some(master_sk)) =
            self.keychain.extract_secrets(wallet.fingerprint, b"")?
        else {
            return Err(Error::NoSigningKey);
        };

        let spend_bundle = wallet
            .sign_take_offer(
                unsigned,
                &AggSigConstants::new(parse_genesis_challenge(self.network().agg_sig_me.clone())?),
                master_sk,
            )
            .await?;

        debug!(
            "{}",
            serde_json::to_string(&json_bundle(&spend_bundle)).expect("msg")
        );

        if req.auto_submit {
            let mut tx = wallet.db.tx().await?;

            let subscriptions = insert_transaction(
                &mut tx,
                spend_bundle.name(),
                Transaction::from_coin_spends(spend_bundle.coin_spends.clone())?,
                spend_bundle.aggregated_signature.clone(),
            )
            .await?;

            tx.commit().await?;

            self.command_sender
                .send(SyncCommand::SubscribeCoins {
                    coin_ids: subscriptions,
                })
                .await?;
        }

        let json_bundle = json_bundle(&spend_bundle);

        Ok(TakeOfferResponse {
            summary: self
                .summarize(spend_bundle.coin_spends, ConfirmationInfo::default())
                .await?,
            spend_bundle: json_bundle,
        })
    }

    pub async fn view_offer(&self, req: ViewOffer) -> Result<ViewOfferResponse> {
        let offer = self.summarize_offer(Offer::decode(&req.offer)?).await?;

        Ok(ViewOfferResponse { offer })
    }

    pub async fn import_offer(&self, req: ImportOffer) -> Result<ImportOfferResponse> {
        let wallet = self.wallet()?;
        let offer = Offer::decode(&req.offer)?;
        let spend_bundle: SpendBundle = offer.clone().into();
        let peer = self.peer_state.lock().await.acquire_peer();

        let mut allocator = Allocator::new();
        let parsed_offer = offer.parse(&mut allocator)?;

        let status = if let Some(peer) = peer {
            let coin_creation = lookup_coin_creation(
                &peer,
                parsed_offer
                    .coin_spends
                    .iter()
                    .map(|cs| cs.coin.coin_id())
                    .collect(),
                parse_genesis_challenge(self.network().genesis_challenge.clone())?,
            )
            .await?;
            offer_expiration(&mut allocator, &parsed_offer, &coin_creation)?
        } else {
            warn!("No peers available to fetch coin creation information, so skipping for now");
            OfferExpiration {
                expiration_height: None,
                expiration_timestamp: None,
            }
        };

        let mut tx = wallet.db.tx().await?;

        let inserted_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before the UNIX epoch")
            .as_secs();

        let offer_id = spend_bundle.name();

        tx.insert_offer(OfferRow {
            offer_id,
            encoded_offer: req.offer,
            expiration_height: status.expiration_height,
            expiration_timestamp: status.expiration_timestamp,
            status: OfferStatus::Active,
            inserted_timestamp,
        })
        .await?;

        for coin_state in parsed_offer.coin_spends {
            tx.insert_offer_coin(offer_id, coin_state.coin.coin_id())
                .await?;
        }

        tx.commit().await?;

        Ok(ImportOfferResponse {})
    }

    pub async fn get_offers(&self, _req: GetOffers) -> Result<GetOffersResponse> {
        let wallet = self.wallet()?;
        let offers = wallet.db.get_offers().await?;

        let mut records = Vec::new();

        for offer in offers {
            records.push(OfferRecord {
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
                    .format("%B %d, %Y %r")
                    .to_string(),
            });
        }

        Ok(GetOffersResponse { offers: records })
    }

    pub async fn delete_offer(&self, req: DeleteOffer) -> Result<DeleteOfferResponse> {
        let wallet = self.wallet()?;
        let offer_id = hex::decode(&req.offer_id)?;

        wallet.db.delete_offer(offer_id.try_into()?).await?;

        Ok(DeleteOfferResponse {})
    }
}
