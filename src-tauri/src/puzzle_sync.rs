use std::{sync::Arc, time::Duration};

use chia::{
    clvm_traits::ToClvm,
    protocol::{Bytes32, CoinState},
};
use chia_wallet_sdk::{Cat, Peer, Primitive, Puzzle};
use clvmr::{Allocator, NodePtr};
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage::Database;
use tauri::Emitter;
use tokio::{sync::Mutex, time::sleep};
use tracing::{debug, warn};

use crate::{
    error::{Error, Result},
    sync_manager::SyncManager,
    wallet::Wallet,
};

pub async fn puzzle_sync(
    wallet: Arc<Wallet>,
    sync_manager: Arc<Mutex<SyncManager>>,
    genesis_challenge: Bytes32,
) {
    loop {
        sleep(Duration::from_secs(3)).await;

        if let Err(error) =
            process_unsynced_puzzles(&wallet, &sync_manager, genesis_challenge).await
        {
            warn!("Error processing unsynced puzzles: {:?}", error);
        }
    }
}

async fn process_unsynced_puzzles(
    wallet: &Wallet,
    sync_manager: &Mutex<SyncManager>,
    genesis_challenge: Bytes32,
) -> Result<()> {
    let peers: Vec<Peer> = sync_manager.lock().await.peers().cloned().collect();
    let coin_states = wallet.db.unsynced_coin_states(peers.len()).await?;

    let mut futures = FuturesUnordered::new();

    for (peer, coin_state) in peers.into_iter().zip(coin_states.into_iter()) {
        debug!(
            "Requesting puzzle and solution for id {} from peer {}",
            coin_state.coin.coin_id(),
            peer.socket_addr()
        );

        futures.push(async move {
            let socket_addr = peer.socket_addr();
            let result = process_coin(peer, wallet.db.clone(), coin_state, genesis_challenge).await;
            (socket_addr, result)
        });
    }

    while let Some((socket_addr, result)) = futures.next().await {
        match result {
            Ok(successful) => {
                if successful {
                    if let Err(error) = wallet.app_handle.emit("coin-update", ()) {
                        warn!("Failed to emit coin update: {error}");
                    }
                } else {
                    warn!("Coin doesn't exist, banning peer");
                    sync_manager.lock().await.ban_peer(&socket_addr);
                }
            }
            Err(error) => {
                warn!("Error processing unsynced puzzle: {:?}", error);
                sync_manager.lock().await.ban_peer(&socket_addr);
            }
        }
    }

    Ok(())
}

async fn process_coin(
    peer: Peer,
    db: Database,
    coin_state: CoinState,
    genesis_challenge: Bytes32,
) -> Result<bool> {
    let Ok(response) = peer
        .request_puzzle_and_solution(
            coin_state.coin.parent_coin_info,
            coin_state.created_height.unwrap(),
        )
        .await?
    else {
        return Ok(false);
    };

    let Ok(parent_response) = peer
        .request_coin_state(
            vec![coin_state.coin.parent_coin_info],
            None,
            genesis_challenge,
            false,
        )
        .await?
    else {
        return Err(Error::ErroneousCoinStateRejection);
    };

    let Some(parent_coin_state) = parent_response.coin_states.into_iter().next() else {
        return Ok(false);
    };

    let mut allocator = Allocator::new();
    let puzzle_ptr = response.puzzle.to_clvm(&mut allocator)?;
    let solution_ptr = response.solution.to_clvm(&mut allocator)?;

    if let Err(error) = add_puzzle_info(
        &db,
        &mut allocator,
        parent_coin_state,
        puzzle_ptr,
        solution_ptr,
        coin_state,
    )
    .await
    {
        warn!("Error fetching puzzle info: {:?}", error);
        db.remove_coin_state(coin_state.coin.coin_id()).await?;
        return Ok(true);
    }

    Ok(true)
}

async fn add_puzzle_info(
    db: &Database,
    allocator: &mut Allocator,
    parent_coin_state: CoinState,
    parent_puzzle_ptr: NodePtr,
    parent_solution: NodePtr,
    coin_state: CoinState,
) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();
    let parent_puzzle = Puzzle::parse(allocator, parent_puzzle_ptr);

    if let Some(cat) = Cat::from_parent_spend(
        allocator,
        parent_coin_state.coin,
        parent_puzzle,
        parent_solution,
        coin_state.coin,
    )? {
        let Some(lineage_proof) = cat.lineage_proof else {
            return Ok(());
        };

        let mut tx = db.tx().await?;

        tx.mark_coin_synced(coin_id).await?;
        tx.insert_cat_info(coin_id, lineage_proof, cat.p2_puzzle_hash, cat.asset_id)
            .await?;

        tx.commit().await?;

        return Ok(());
    }

    Err(Error::UnknownPuzzle(
        coin_id,
        parent_puzzle.mod_hash().into(),
    ))
}
