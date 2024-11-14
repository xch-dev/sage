use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin},
    puzzles::offer::SETTLEMENT_PAYMENTS_PUZZLE_HASH,
};
use chia_wallet_sdk::{
    decode_address, encode_address, run_puzzle, AggSigConstants, Condition, Conditions, Offer,
    Puzzle, MAINNET_CONSTANTS, TESTNET11_CONSTANTS,
};
use clvmr::Allocator;
use indexmap::IndexMap;
use sage_api::{
    Amount, AssetKind, CatAmount, MakeOffer, OfferSummary, OfferedCoin, RequestedAsset,
};
use sage_wallet::{
    fetch_nft_offer_details, insert_transaction, ChildKind, CoinKind, MakerSide, SyncCommand,
    TakerSide, Transaction, Wallet,
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
            &if state.config.network.network_id == "mainnet" {
                AggSigConstants::new(MAINNET_CONSTANTS.agg_sig_me_additional_data)
            } else {
                AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data)
            },
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
            &if state.config.network.network_id == "mainnet" {
                AggSigConstants::new(MAINNET_CONSTANTS.agg_sig_me_additional_data)
            } else {
                AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data)
            },
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
    let mut allocator = Allocator::new();
    let offer = offer.parse(&mut allocator)?;

    let mut offered = Vec::new();
    let mut fee = 0;

    for coin_spend in &offer.coin_spends {
        let parent_coin = coin_spend.coin;
        let parent_puzzle = coin_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle);
        let parent_solution = coin_spend.solution.to_clvm(&mut allocator)?;

        let output = run_puzzle(&mut allocator, parent_puzzle.ptr(), parent_solution)?;
        let conditions = Conditions::from_clvm(&allocator, output)?;

        let mut coins = Vec::new();

        for condition in conditions.clone() {
            match condition {
                Condition::CreateCoin(cond) => coins.push(Coin::new(
                    parent_coin.coin_id(),
                    cond.puzzle_hash,
                    cond.amount,
                )),
                Condition::ReserveFee(cond) => fee += cond.amount,
                _ => {}
            }
        }

        let mut offered_amount = 0;
        let mut kind = AssetKind::Unknown;

        for coin in coins {
            if matches!(kind, AssetKind::Unknown)
                && coin.puzzle_hash == SETTLEMENT_PAYMENTS_PUZZLE_HASH.into()
            {
                kind = AssetKind::Xch;
            }

            let child = ChildKind::from_parent_cached(
                &mut allocator,
                parent_coin,
                parent_puzzle,
                parent_solution,
                conditions.clone().into_iter().collect(),
                coin,
            )?;

            if child.p2_puzzle_hash() != Some(SETTLEMENT_PAYMENTS_PUZZLE_HASH.into())
                && coin.puzzle_hash != SETTLEMENT_PAYMENTS_PUZZLE_HASH.into()
            {
                continue;
            }

            offered_amount += coin.amount;

            match child {
                ChildKind::Launcher | ChildKind::Unknown { .. } => {}
                ChildKind::Cat { asset_id, .. } => {
                    // TODO: Melt?

                    let cat = wallet.db.cat(asset_id).await?;

                    kind = AssetKind::Cat {
                        asset_id: hex::encode(asset_id),
                        name: cat.as_ref().and_then(|cat| cat.name.clone()),
                        ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                        icon_url: cat.as_ref().and_then(|cat| cat.icon.clone()),
                    };
                }
                ChildKind::Did { info, .. } => {
                    let name = wallet.db.did_name(info.launcher_id).await?;

                    kind = AssetKind::Did {
                        launcher_id: encode_address(info.launcher_id.into(), "did:chia:")?,
                        name,
                    };
                }
                ChildKind::Nft { info, metadata, .. } => {
                    let extracted =
                        extract_nft_data(Some(&wallet.db), metadata, &ConfirmationInfo::default())
                            .await?;

                    kind = AssetKind::Nft {
                        launcher_id: encode_address(info.launcher_id.into(), "nft")?,
                        image_data: extracted.image_data,
                        image_mime_type: extracted.image_mime_type,
                        name: extracted.name,
                    };
                }
            }
        }

        offered.push(OfferedCoin {
            coin_id: hex::encode(parent_coin.coin_id()),
            offered_amount: Amount::from_mojos(
                offered_amount as u128,
                if matches!(kind, AssetKind::Cat { .. }) {
                    3
                } else {
                    state.unit.decimals
                },
            ),
            kind,
        });
    }

    let mut requested = Vec::new();

    for (_hash, (puzzle, payments)) in offer.requested_payments {
        let kind = CoinKind::from_puzzle_cached(&allocator, puzzle)?;

        let payments: u64 = payments
            .into_iter()
            .flat_map(|item| item.payments)
            .map(|payment| payment.amount)
            .sum();

        let kind = match kind {
            CoinKind::Unknown => {
                if puzzle.curried_puzzle_hash() == SETTLEMENT_PAYMENTS_PUZZLE_HASH {
                    AssetKind::Xch
                } else {
                    AssetKind::Unknown
                }
            }
            CoinKind::Launcher => AssetKind::Launcher,
            CoinKind::Cat { asset_id, .. } => {
                let cat = wallet.db.cat(asset_id).await?;

                AssetKind::Cat {
                    asset_id: hex::encode(asset_id),
                    name: cat.as_ref().and_then(|cat| cat.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.icon.clone()),
                }
            }
            CoinKind::Did { info } => {
                let name = wallet.db.did_name(info.launcher_id).await?;

                AssetKind::Did {
                    launcher_id: encode_address(info.launcher_id.into(), "did:chia:")?,
                    name,
                }
            }
            CoinKind::Nft { info, metadata } => {
                let extracted =
                    extract_nft_data(Some(&wallet.db), metadata, &ConfirmationInfo::default())
                        .await?;

                AssetKind::Nft {
                    launcher_id: encode_address(info.launcher_id.into(), "nft")?,
                    image_data: extracted.image_data,
                    image_mime_type: extracted.image_mime_type,
                    name: extracted.name,
                }
            }
        };

        requested.push(RequestedAsset {
            amount: Amount::from_mojos(
                payments as u128,
                if matches!(kind, AssetKind::Cat { .. }) {
                    3
                } else {
                    state.unit.decimals
                },
            ),
            kind,
        });
    }

    Ok(OfferSummary {
        fee: Amount::from_mojos(fee as u128, state.unit.decimals),
        offered,
        requested,
    })
}
