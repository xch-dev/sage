use std::sync::Arc;

use chia::{
    bls::DerivableKey,
    protocol::{Bytes32, CoinStateFilters, RejectStateReason},
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{Network, Peer};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::{sync::Mutex, task::spawn_blocking};
use tracing::{debug, info, instrument, warn};

use crate::{
    error::{Error, Result},
    wallet::Wallet,
};

use super::sync_state::SyncState;

pub async fn sync_wallet(wallet: Arc<Wallet>, network: Network, peer: Peer) -> Result<()> {
    info!("Starting sync against peer {}", peer.socket_addr());

    let mut tx = wallet.db.tx().await?;
    let p2_puzzle_hashes = tx.p2_puzzle_hashes().await?;
    let (start_height, start_header_hash) = tx.latest_peak().await?.map_or_else(
        || (None, network.genesis_challenge),
        |(peak, header_hash)| (Some(peak), header_hash),
    );
    tx.commit().await?;

    let mut derive_more = true;

    for batch in p2_puzzle_hashes.chunks(1000) {
        derive_more =
            sync_puzzle_hashes(&wallet, &peer, start_height, start_header_hash, batch).await?;
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
            derive_more =
                sync_puzzle_hashes(&wallet, &peer, None, network.genesis_challenge, batch).await?;
        }

        start_index += new_derivations.len() as u32;

        let mut tx = wallet.db.tx().await?;
        for (index, synthetic_key, p2_puzzle_hash) in new_derivations {
            tx.insert_derivation(p2_puzzle_hash, index, false, synthetic_key)
                .await?;
        }
        tx.commit().await?;
    }

    Ok(())
}

pub async fn lookup_puzzles(wallet: Arc<Wallet>, state: Arc<Mutex<SyncState>>) {}

#[instrument(skip(wallet, peer, puzzle_hashes))]
async fn sync_puzzle_hashes(
    wallet: &Wallet,
    peer: &Peer,
    start_height: Option<u32>,
    start_header_hash: Bytes32,
    puzzle_hashes: &[Bytes32],
) -> Result<bool> {
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

        let response = peer
            .request_puzzle_state(
                puzzle_hashes.to_vec(),
                prev_height,
                prev_header_hash,
                CoinStateFilters::new(true, true, true, 0),
                true,
            )
            .await?;

        match response {
            Ok(data) => {
                debug!("Received {} coin states", data.coin_states.len());

                let mut tx = wallet.db.tx().await?;

                for coin_state in data.coin_states {
                    found_coins = true;

                    tx.try_insert_coin_state(coin_state).await?;

                    if tx.is_p2_puzzle_hash(coin_state.coin.puzzle_hash).await? {
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

    Ok(found_coins)
}
