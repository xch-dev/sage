use std::{sync::Arc, time::Duration};

use chia::{
    bls::DerivableKey,
    clvm_traits::ToClvm,
    protocol::{Bytes32, CoinState, CoinStateFilters, RejectStateReason},
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{Cat, Peer, Primitive, Puzzle};
use clvmr::Allocator;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::{
    sync::Mutex,
    task::spawn_blocking,
    time::{sleep, timeout},
};
use tracing::{debug, info, instrument, warn};

use crate::{
    error::{Error, Result},
    wallet::Wallet,
};

use super::sync_state::SyncState;

pub async fn sync_wallet(
    wallet: Arc<Wallet>,
    genesis_challenge: Bytes32,
    peer: Peer,
) -> Result<()> {
    info!("Starting sync against peer {}", peer.socket_addr());

    let mut tx = wallet.db.tx().await?;
    let p2_puzzle_hashes = tx.p2_puzzle_hashes().await?;
    let (start_height, start_header_hash) = tx.latest_peak().await?.map_or_else(
        || (None, genesis_challenge),
        |(peak, header_hash)| (Some(peak), header_hash),
    );
    tx.commit().await?;

    let mut derive_more = true;

    let mut new_peak = None;

    for batch in p2_puzzle_hashes.chunks(1000) {
        let sync =
            sync_puzzle_hashes(&wallet, &peer, start_height, start_header_hash, batch).await?;

        derive_more = sync.found_coins;

        if new_peak.is_none() {
            new_peak = Some((sync.prev_height, sync.prev_header_hash));
        }
    }

    let mut start_index = p2_puzzle_hashes.len() as u32;

    while derive_more {
        let intermediate_pk = wallet.intermediate_pk;

        let new_derivations = spawn_blocking(move || {
            (start_index..start_index + 1000)
                .into_par_iter()
                .map(|index| {
                    let synthetic_key = intermediate_pk.derive_unhardened(index).derive_synthetic();
                    let p2_puzzle_hash =
                        Bytes32::from(StandardArgs::curry_tree_hash(synthetic_key));
                    (index, synthetic_key, p2_puzzle_hash)
                })
                .collect::<Vec<_>>()
        })
        .await?;

        let p2_puzzle_hashes: Vec<Bytes32> = new_derivations
            .iter()
            .map(|(_, _, p2_puzzle_hash)| *p2_puzzle_hash)
            .collect();

        for batch in p2_puzzle_hashes.chunks(1000) {
            let sync = sync_puzzle_hashes(&wallet, &peer, None, genesis_challenge, batch).await?;

            derive_more = sync.found_coins;

            if new_peak.is_none() {
                new_peak = Some((sync.prev_height, sync.prev_header_hash));
            }
        }

        start_index += new_derivations.len() as u32;

        let mut tx = wallet.db.tx().await?;
        for (index, synthetic_key, p2_puzzle_hash) in new_derivations {
            tx.insert_derivation(p2_puzzle_hash, index, false, synthetic_key)
                .await?;
        }
        if let Some((Some(height), header_hash)) = new_peak {
            tx.insert_peak(height, header_hash).await?;
        }
        tx.commit().await?;
    }

    Ok(())
}

enum PuzzleInfo {
    Cat(Box<Cat>),
    Unknown,
}

pub async fn lookup_puzzles(
    wallet: Arc<Wallet>,
    genesis_challenge: Bytes32,
    state: Arc<Mutex<SyncState>>,
) -> Result<()> {
    loop {
        let coin_states = wallet.db.unsynced_coin_states(30).await?;

        if coin_states.is_empty() {
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        let peers: Vec<Peer> = state.lock().await.peers().cloned().collect();

        info!(
            "Looking up puzzles for {} coins",
            coin_states.len().min(peers.len())
        );

        let mut futures = FuturesUnordered::new();

        for (peer, coin_state) in peers.into_iter().zip(coin_states.into_iter()) {
            let wallet = wallet.clone();
            let addr = peer.socket_addr();
            let coin_id = coin_state.coin.coin_id();
            futures.push(tokio::spawn(async move {
                let result = lookup_puzzle(wallet, peer, genesis_challenge, coin_state).await;
                (addr, coin_id, result)
            }));
        }

        while let Some(result) = futures.next().await {
            let (addr, coin_id, result) = result?;

            if let Err(error) = result {
                warn!(
                    "Failed to lookup puzzle for coin {} from peer {}: {}",
                    coin_id, addr, error
                );
                state.lock().await.ban(addr.ip());
            } else {
                wallet.db.mark_coin_synced(coin_id).await?;
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}

async fn lookup_puzzle(
    wallet: Arc<Wallet>,
    peer: Peer,
    genesis_challenge: Bytes32,
    coin_state: CoinState,
) -> Result<()> {
    let Some(parent_coin_state) = timeout(
        Duration::from_secs(2),
        peer.request_coin_state(
            vec![coin_state.coin.parent_coin_info],
            None,
            genesis_challenge,
            false,
        ),
    )
    .await??
    .map_err(|_| Error::Rejection)?
    .coin_states
    .into_iter()
    .next() else {
        return Err(Error::CoinStateNotFound);
    };

    let height = coin_state
        .created_height
        .ok_or(Error::MissingCreatedHeight)?;

    let response = timeout(
        Duration::from_secs(3),
        peer.request_puzzle_and_solution(coin_state.coin.parent_coin_info, height),
    )
    .await??
    .map_err(|_| Error::Rejection)?;

    let info = spawn_blocking(move || -> Result<PuzzleInfo> {
        let mut allocator = Allocator::new();

        let parent_puzzle_ptr = response.puzzle.to_clvm(&mut allocator)?;
        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle_ptr);

        let parent_solution = response.solution.to_clvm(&mut allocator)?;

        if let Some(cat) = Cat::from_parent_spend(
            &mut allocator,
            parent_coin_state.coin,
            parent_puzzle,
            parent_solution,
            coin_state.coin,
        )
        .ok()
        .flatten()
        {
            Ok(PuzzleInfo::Cat(Box::new(cat)))
        } else {
            Ok(PuzzleInfo::Unknown)
        }
    })
    .await??;

    let mut tx = wallet.db.tx().await?;

    match info {
        PuzzleInfo::Cat(cat) => {
            if let Some(lineage_proof) = cat.lineage_proof {
                tx.insert_cat_coin(
                    cat.coin.coin_id(),
                    lineage_proof,
                    cat.p2_puzzle_hash,
                    cat.asset_id,
                )
                .await?;
            }
        }
        PuzzleInfo::Unknown => {
            // TODO: tx.insert_unknown_coin(coin_state.coin.coin_id()).await?;
        }
    }

    tx.mark_coin_synced(coin_state.coin.coin_id()).await?;

    tx.commit().await?;

    Ok(())
}

struct Sync {
    found_coins: bool,
    prev_height: Option<u32>,
    prev_header_hash: Bytes32,
}

#[instrument(skip(wallet, peer, puzzle_hashes))]
async fn sync_puzzle_hashes(
    wallet: &Wallet,
    peer: &Peer,
    start_height: Option<u32>,
    start_header_hash: Bytes32,
    puzzle_hashes: &[Bytes32],
) -> Result<Sync> {
    let mut prev_height = start_height;
    let mut prev_header_hash = start_header_hash;
    let mut found_coins = false;

    loop {
        debug!(
            "Requesting puzzle state from previous height {:?} with header hash {} from peer {}",
            prev_height,
            prev_header_hash,
            peer.socket_addr()
        );

        let response = timeout(
            Duration::from_secs(5),
            peer.request_puzzle_state(
                puzzle_hashes.to_vec(),
                prev_height,
                prev_header_hash,
                CoinStateFilters::new(true, true, true, 0),
                true,
            ),
        )
        .await??;

        match response {
            Ok(data) => {
                debug!("Received {} coin states", data.coin_states.len());

                let mut tx = wallet.db.tx().await?;

                for coin_state in data.coin_states {
                    found_coins = true;

                    let is_p2 = tx.is_p2_puzzle_hash(coin_state.coin.puzzle_hash).await?;

                    tx.insert_coin_state(coin_state, is_p2).await?;

                    if is_p2 {
                        tx.insert_p2_coin(coin_state.coin.coin_id()).await?;
                    }
                }

                tx.commit().await?;

                if data.is_finished {
                    break;
                }

                prev_height = Some(data.height);
                prev_header_hash = data.header_hash;
            }
            Err(rejection) => match rejection.reason {
                RejectStateReason::ExceededSubscriptionLimit => {
                    warn!(
                        "Subscription limit reached against peer {}",
                        peer.socket_addr()
                    );
                    return Err(Error::SubscriptionLimitReached);
                }
                RejectStateReason::Reorg => {
                    // TODO: Handle reorgs gracefully
                    todo!();
                }
            },
        }
    }

    Ok(Sync {
        found_coins,
        prev_height,
        prev_header_hash,
    })
}
