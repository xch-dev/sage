use bigdecimal::BigDecimal;
use chia::{clvm_traits::FromClvm, protocol::Bytes32, puzzles::nft::NftMetadata};
use chia_wallet_sdk::{decode_address, encode_address, AggSigConstants, Offer, SpendContext};
use indexmap::IndexMap;
use sage_api::{
    Amount, CatAmount, MakeOffer, OfferAssets, OfferCat, OfferNft, OfferSummary, OfferXch,
};
use sage_wallet::{
    calculate_royalties, fetch_nft_offer_details, insert_transaction, parse_locked_coins,
    parse_offer_payments, MakerSide, NftRoyaltyInfo, SyncCommand, TakerSide, Transaction, Wallet,
};
use specta::specta;
use tauri::{command, State};
use tokio::sync::MutexGuard;

use crate::{
    app_state::{AppState, AppStateInner},
    error::{Error, Result},
};

use super::{extract_nft_data, ConfirmationInfo};

#[command]
#[specta]
pub async fn make_offer(state: State<'_, AppState>, request: MakeOffer) -> Result<String> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let Some(offered_xch) = request.offered_assets.xch.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&request.offered_assets.xch));
    };

    let mut offered_cats = IndexMap::new();

    for CatAmount { asset_id, amount } in request.offered_assets.cats {
        let Some(amount) = amount.to_mojos(3) else {
            return Err(Error::invalid_amount(&amount));
        };
        let asset_id = hex::decode(&asset_id)?.try_into()?;
        offered_cats.insert(asset_id, amount);
    }

    let mut offered_nfts = Vec::new();

    for nft_id in request.offered_assets.nfts {
        let (launcher_id, prefix) = decode_address(&nft_id)?;

        if prefix != "nft" {
            return Err(Error::invalid_prefix(&prefix));
        }

        offered_nfts.push(launcher_id.into());
    }

    let Some(requested_xch) = request.requested_assets.xch.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&request.requested_assets.xch));
    };

    let mut requested_cats = IndexMap::new();

    for CatAmount { asset_id, amount } in request.requested_assets.cats {
        let Some(amount) = amount.to_mojos(3) else {
            return Err(Error::invalid_amount(&amount));
        };
        let asset_id = hex::decode(&asset_id)?.try_into()?;
        requested_cats.insert(asset_id, amount);
    }

    let mut requested_nfts = IndexMap::new();
    let mut peer = None;

    for nft_id in request.requested_assets.nfts {
        if peer.is_none() {
            peer = state.peer_state.lock().await.acquire_peer();
        }

        let peer = peer.as_ref().ok_or(Error::no_peers())?;

        let (launcher_id, prefix) = decode_address(&nft_id)?;

        if prefix != "nft" {
            return Err(Error::invalid_prefix(&prefix));
        }

        let nft_id: Bytes32 = launcher_id.into();
        let Some(offer_details) = fetch_nft_offer_details(peer, nft_id).await? else {
            return Err(Error::invalid_launcher_id());
        };

        requested_nfts.insert(nft_id, offer_details);
    }

    let Some(fee) = request.fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&request.fee));
    };

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

    let (_mnemonic, Some(master_sk)) = state.keychain.extract_secrets(wallet.fingerprint, b"")?
    else {
        return Err(Error::no_secret_key());
    };

    let offer = wallet
        .sign_make_offer(
            unsigned,
            &AggSigConstants::new(hex::decode(&state.network().agg_sig_me)?.try_into()?),
            master_sk,
        )
        .await?;

    Ok(offer.encode()?)
}

#[command]
#[specta]
pub async fn take_offer(state: State<'_, AppState>, offer: String, fee: Amount) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let offer = Offer::decode(&offer)?;

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let unsigned = wallet.take_offer(offer, fee, false, true).await?;

    let (_mnemonic, Some(master_sk)) = state.keychain.extract_secrets(wallet.fingerprint, b"")?
    else {
        return Err(Error::no_secret_key());
    };

    let spend_bundle = wallet
        .sign_take_offer(
            unsigned,
            &AggSigConstants::new(hex::decode(&state.network().agg_sig_me)?.try_into()?),
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

    state
        .command_sender
        .send(SyncCommand::SubscribeCoins {
            coin_ids: subscriptions,
        })
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn view_offer(state: State<'_, AppState>, offer: String) -> Result<OfferSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    summarize_offer(&state, &wallet, Offer::decode(&offer)?).await
}

async fn summarize_offer(
    state: &MutexGuard<'_, AppStateInner>,
    wallet: &Wallet,
    offer: Offer,
) -> Result<OfferSummary> {
    let mut ctx = SpendContext::new();

    let offer = offer.parse(&mut ctx.allocator)?;
    let locked_coins = parse_locked_coins(&mut ctx.allocator, &offer)?;
    let maker_amounts = locked_coins.amounts();

    let mut builder = offer.take();
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
            amount: Amount::from_mojos(maker_amounts.xch as u128, state.unit.decimals),
            royalty: Amount::from_mojos(maker_royalties.xch as u128, state.unit.decimals),
        },
        cats: IndexMap::new(),
        nfts: IndexMap::new(),
    };

    for (asset_id, amount) in maker_amounts.cats {
        let cat = wallet.db.cat(asset_id).await?;

        maker.cats.insert(
            hex::encode(asset_id),
            OfferCat {
                amount: Amount::from_mojos(amount as u128, 3),
                royalty: Amount::from_mojos(
                    maker_royalties.cats.get(&asset_id).copied().unwrap_or(0) as u128,
                    3,
                ),
                name: cat.as_ref().and_then(|cat| cat.name.clone()),
                ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                icon_url: cat.as_ref().and_then(|cat| cat.icon.clone()),
            },
        );
    }

    for (launcher_id, nft) in locked_coins.nfts {
        let metadata = NftMetadata::from_clvm(&ctx.allocator, nft.info.metadata.ptr())?;
        let info = extract_nft_data(
            Some(&wallet.db),
            Some(metadata),
            &ConfirmationInfo::default(),
        )
        .await?;

        maker.nfts.insert(
            encode_address(launcher_id.to_bytes(), "nft")?,
            OfferNft {
                image_data: info.image_data,
                image_mime_type: info.image_mime_type,
                name: info.name,
                royalty_percent: (BigDecimal::from(nft.info.royalty_ten_thousandths)
                    / BigDecimal::from(100))
                .to_string(),
            },
        );
    }

    let mut taker = OfferAssets {
        xch: OfferXch {
            amount: Amount::from_mojos(taker_amounts.xch as u128, state.unit.decimals),
            royalty: Amount::from_mojos(taker_royalties.xch as u128, state.unit.decimals),
        },
        cats: IndexMap::new(),
        nfts: IndexMap::new(),
    };

    for (asset_id, amount) in taker_amounts.cats {
        let cat = wallet.db.cat(asset_id).await?;

        taker.cats.insert(
            hex::encode(asset_id),
            OfferCat {
                amount: Amount::from_mojos(amount as u128, 3),
                royalty: Amount::from_mojos(
                    taker_royalties.cats.get(&asset_id).copied().unwrap_or(0) as u128,
                    3,
                ),
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
                royalty_percent: (BigDecimal::from(nft.royalty_ten_thousandths)
                    / BigDecimal::from(100))
                .to_string(),
            },
        );
    }

    Ok(OfferSummary {
        fee: Amount::from_mojos(locked_coins.fee as u128, state.unit.decimals),
        maker,
        taker,
    })
}
