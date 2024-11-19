use chia_wallet_sdk::{AggSigConstants, Offer};
use indexmap::IndexMap;
use sage_api::{
    CatAmount, MakeOffer, MakeOfferResponse, TakeOffer, TakeOfferResponse, ViewOffer,
    ViewOfferResponse,
};
use sage_wallet::{
    fetch_nft_offer_details, insert_transaction, MakerSide, SyncCommand, TakerSide, Transaction,
};

use crate::{
    parse_asset_id, parse_cat_amount, parse_genesis_challenge, parse_nft_id, Error, Result, Sage,
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

        let mut tx = wallet.db.tx().await?;

        let subscriptions = insert_transaction(
            &mut tx,
            spend_bundle.name(),
            Transaction::from_coin_spends(spend_bundle.coin_spends)?,
            spend_bundle.aggregated_signature,
        )
        .await?;

        tx.commit().await?;

        self.command_sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await?;

        Ok(TakeOfferResponse {})
    }

    pub async fn view_offer(&self, req: ViewOffer) -> Result<ViewOfferResponse> {
        let offer = self.summarize_offer(Offer::decode(&req.offer)?).await?;

        Ok(ViewOfferResponse { offer })
    }
}
