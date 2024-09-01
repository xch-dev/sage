use std::{sync::Arc, time::Duration};

use chia::protocol::{Bytes32, CoinState};
use chia_wallet_sdk::Peer;
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage_database::Database;
use tokio::{sync::Mutex, task::spawn_blocking, time::timeout};
use tracing::{info, instrument, warn};

use crate::{PuzzleInfo, SyncError, WalletError};

use super::PeerState;

#[derive(Debug)]
pub struct PuzzleQueue {
    db: Database,
    genesis_challenge: Bytes32,
    state: Arc<Mutex<PeerState>>,
}

impl PuzzleQueue {
    pub fn new(db: Database, genesis_challenge: Bytes32, state: Arc<Mutex<PeerState>>) -> Self {
        Self {
            db,
            genesis_challenge,
            state,
        }
    }

    pub async fn start(mut self) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        let peers: Vec<Peer> = self.state.lock().await.peers().cloned().collect();
        if peers.is_empty() {
            return Ok(());
        }

        let coin_states = self.db.unsynced_coin_states(peers.len()).await?;
        if coin_states.is_empty() {
            return Ok(());
        }

        info!(
            "Syncing a batch of {} coins",
            coin_states.len().min(peers.len())
        );

        let mut futures = FuturesUnordered::new();

        for (peer, coin_state) in peers.into_iter().zip(coin_states.into_iter()) {
            let db = self.db.clone();
            let genesis_challenge = self.genesis_challenge;
            let addr = peer.socket_addr();
            let coin_id = coin_state.coin.coin_id();
            futures.push(tokio::spawn(async move {
                let result = fetch_puzzle(&peer, &db, genesis_challenge, coin_state).await;
                (addr, coin_id, result)
            }));
        }

        while let Some(result) = futures.next().await {
            let (addr, coin_id, result) = result?;

            if let Err(error) = result {
                // TODO: Not all errors should result in this exact behavior.
                warn!(
                    "Failed to lookup puzzle of {} from peer {}: {}",
                    coin_id, addr, error
                );
                self.state.lock().await.ban(addr.ip());
            }
        }

        Ok(())
    }
}

/// Fetches info for a coin's puzzle and inserts it into the database.
#[instrument(skip(peer, db))]
async fn fetch_puzzle(
    peer: &Peer,
    db: &Database,
    genesis_challenge: Bytes32,
    coin_state: CoinState,
) -> Result<(), WalletError> {
    let parent_id = coin_state.coin.parent_coin_info;

    let Some(parent_coin_state) = timeout(
        Duration::from_secs(3),
        peer.request_coin_state(vec![parent_id], None, genesis_challenge, false),
    )
    .await
    .map_err(|_| SyncError::Timeout)??
    .map_err(|_| SyncError::Rejection)?
    .coin_states
    .into_iter()
    .next() else {
        return Err(SyncError::MissingCoinState(parent_id).into());
    };

    let height = coin_state
        .created_height
        .ok_or(SyncError::UnconfirmedCoin(parent_id))?;

    let response = timeout(
        Duration::from_secs(3),
        peer.request_puzzle_and_solution(parent_id, height),
    )
    .await
    .map_err(|_| SyncError::Timeout)??
    .map_err(|_| SyncError::MissingPuzzleAndSolution(parent_id))?;

    let info = spawn_blocking(move || {
        PuzzleInfo::parse(
            parent_coin_state.coin,
            &response.puzzle,
            &response.solution,
            coin_state.coin,
        )
    })
    .await??;

    let coin_id = coin_state.coin.coin_id();

    let mut tx = db.tx().await?;

    match info {
        PuzzleInfo::Cat {
            asset_id,
            lineage_proof,
            p2_puzzle_hash,
        } => {
            tx.mark_coin_synced(coin_id, p2_puzzle_hash).await?;
            tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                .await?;
        }
        PuzzleInfo::Did {
            lineage_proof,
            info,
        } => {
            tx.mark_coin_synced(coin_id, info.p2_puzzle_hash).await?;
            tx.insert_did_coin(coin_id, lineage_proof, info).await?;
        }
        PuzzleInfo::Nft {
            lineage_proof,
            info,
        } => {
            tx.mark_coin_synced(coin_id, info.p2_puzzle_hash).await?;
            tx.insert_nft_coin(coin_id, lineage_proof, info).await?;
        }
        PuzzleInfo::Unknown { hint } => {
            tx.mark_coin_synced(coin_id, hint).await?;
            tx.insert_unknown_coin(coin_id).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}
