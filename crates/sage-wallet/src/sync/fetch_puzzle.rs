use std::time::Duration;

use chia::protocol::{Bytes32, CoinState};
use chia_wallet_sdk::Peer;
use sage_database::Database;
use tokio::{task::spawn_blocking, time::timeout};
use tracing::instrument;

use crate::{PuzzleInfo, SyncError, WalletError};

/// Fetches info for a coin's puzzle and inserts it into the database.
#[instrument(skip(peer, db))]
pub async fn fetch_puzzle(
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
            tx.mark_coin_synced(coin_id, Some(p2_puzzle_hash)).await?;
            tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                .await?;
        }
        PuzzleInfo::Did {
            lineage_proof,
            info,
        } => {
            tx.mark_coin_synced(coin_id, Some(info.p2_puzzle_hash))
                .await?;
            tx.insert_did_coin(coin_id, lineage_proof, info).await?;
        }
        PuzzleInfo::Nft {
            lineage_proof,
            info,
        } => {
            tx.mark_coin_synced(coin_id, Some(info.p2_puzzle_hash))
                .await?;
            tx.insert_nft_coin(coin_id, lineage_proof, info).await?;
        }
        PuzzleInfo::Unknown => {
            tx.mark_coin_synced(coin_id, None).await?;
            tx.insert_unknown_coin(coin_id).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}
