use chia::{bls::Signature, protocol::SpendBundle, puzzles::nft::NftMetadata};
use chia_wallet_sdk::Offer;
use indexmap::IndexMap;
use sage_api::{CatAmount, MintOption, TransactionResponse};
use sage_wallet::{fetch_nft_offer_details, MakerSide, Option, TakerSide};
use tracing::debug;

use crate::{parse_asset_id, parse_cat_amount, parse_did_id, parse_nft_id, Error, Result, Sage};

impl Sage {
    pub async fn mint_option(&self, req: MintOption) -> Result<TransactionResponse> {
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
        let did_id = parse_did_id(req.did_id)?;

        let coin_spends = wallet
            .mint_option(
                Option {
                    maker: MakerSide {
                        xch: offered_xch,
                        cats: offered_cats,
                        nfts: offered_nfts,
                        fee,
                    },
                    taker: TakerSide {
                        xch: requested_xch,
                        cats: requested_cats,
                        nfts: requested_nfts,
                    },
                    expiration_seconds: req.expires_at_second,
                    nft_metadata: NftMetadata::default(),
                    did_id,
                },
                false,
                true,
            )
            .await?;

        debug!(
            "coin_spends: {:?}",
            Offer::from(SpendBundle::new(coin_spends.clone(), Signature::default())).encode()?
        );

        self.transact(coin_spends, req.auto_submit).await
    }
}
