use std::{sync::Arc, time::Duration};

use chia::{
    clvm_traits::ToNodePtr,
    protocol::{Bytes32, CoinState},
};
use chia_wallet_sdk::{CatPuzzle, Puzzle};
use clvmr::Allocator;
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage::Database;
use sage_client::Peer;
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
            let result = process_coin(peer, wallet.db.clone(), coin_state, genesis_challenge).await;
            (coin_state.coin.coin_id(), result)
        });
    }

    while let Some((coin_id, result)) = futures.next().await {
        if let Err(error) = result {
            warn!("Error processing unsynced puzzle: {:?}", error);
            wallet.db.remove_coin_state(coin_id).await?;
        }
    }

    Ok(())
}

async fn process_coin(
    peer: Peer,
    db: Database,
    coin_state: CoinState,
    genesis_challenge: Bytes32,
) -> Result<()> {
    let coin_id = coin_state.coin.coin_id();

    let response = peer
        .request_puzzle_and_solution(
            coin_state.coin.parent_coin_info,
            coin_state.created_height.unwrap(),
        )
        .await?
        .map_err(|_| Error::RejectPuzzleSolution(coin_id))?;

    let mut allocator = Allocator::new();
    let puzzle_ptr = response.puzzle.to_node_ptr(&mut allocator)?;
    let solution_ptr = response.solution.to_node_ptr(&mut allocator)?;

    let puzzle = Puzzle::parse(&allocator, puzzle_ptr);

    if let Some(cat) = CatPuzzle::parse(&allocator, &puzzle)? {
        let response = peer
            .request_coin_state(
                vec![coin_state.coin.parent_coin_info],
                None,
                genesis_challenge,
                false,
            )
            .await?
            .map_err(|_| Error::RejectCoinState(coin_id))?;

        let parent_coin_state = response
            .coin_states
            .into_iter()
            .next()
            .ok_or_else(|| Error::MissingCoinState(coin_state.coin.parent_coin_info))?;

        let cat_info = cat.child_coin_info(
            &mut allocator,
            parent_coin_state.coin,
            coin_state.coin,
            solution_ptr,
        )?;

        let mut tx = db.tx().await?;

        tx.mark_coin_synced(coin_id).await?;
        tx.insert_cat_info(
            coin_id,
            cat_info.lineage_proof,
            cat_info.p2_puzzle_hash,
            cat_info.asset_id,
        )
        .await?;

        tx.commit().await?;

        return Ok(());
    }

    Err(Error::UnknownCoinType(
        coin_state.coin.coin_id(),
        puzzle.mod_hash().into(),
    ))
}
