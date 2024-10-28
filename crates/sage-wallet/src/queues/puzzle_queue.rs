use std::{sync::Arc, time::Duration};

use chia::protocol::{Bytes32, CoinState};
use chia_wallet_sdk::Peer;
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage_database::Database;
use tokio::{
    sync::{mpsc, Mutex},
    task::spawn_blocking,
    time::{sleep, timeout},
};
use tracing::{debug, instrument};

use crate::{ChildKind, PeerState, SyncCommand, SyncError, SyncEvent, WalletError};

#[derive(Debug)]
pub struct PuzzleQueue {
    db: Database,
    genesis_challenge: Bytes32,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
}

impl PuzzleQueue {
    pub fn new(
        db: Database,
        genesis_challenge: Bytes32,
        state: Arc<Mutex<PeerState>>,
        sync_sender: mpsc::Sender<SyncEvent>,
        command_sender: mpsc::Sender<SyncCommand>,
    ) -> Self {
        Self {
            db,
            genesis_challenge,
            state,
            sync_sender,
            command_sender,
        }
    }

    pub async fn start(mut self) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(Duration::from_millis(150)).await;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        let peers: Vec<Peer> = self
            .state
            .lock()
            .await
            .peers()
            .map(|info| info.peer.clone())
            .collect();

        if peers.is_empty() {
            sleep(Duration::from_secs(3)).await;
            return Ok(());
        }

        let coin_states = self.db.unsynced_coin_states(peers.len()).await?;
        if coin_states.is_empty() {
            sleep(Duration::from_secs(3)).await;
            return Ok(());
        }

        debug!(
            "Syncing a batch of {} coins",
            coin_states.len().min(peers.len())
        );

        let mut futures = FuturesUnordered::new();

        for (peer, coin_state) in peers.into_iter().zip(coin_states.into_iter()) {
            let db = self.db.clone();
            let genesis_challenge = self.genesis_challenge;
            let addr = peer.socket_addr();
            let coin_id = coin_state.coin.coin_id();
            let command_sender = self.command_sender.clone();

            futures.push(tokio::spawn(async move {
                let result =
                    fetch_puzzle(&peer, &db, genesis_challenge, coin_state, command_sender).await;
                (addr, coin_id, result)
            }));
        }

        while let Some(result) = futures.next().await {
            let (addr, coin_id, result) = result?;

            if let Err(error) = result {
                // TODO: Not all errors should result in this exact behavior.
                debug!(
                    "Failed to lookup puzzle of {} from peer {}: {}",
                    coin_id, addr, error
                );

                self.state
                    .lock()
                    .await
                    .ban(addr.ip(), Duration::from_secs(300));
            }
        }

        self.sync_sender
            .send(SyncEvent::PuzzleBatchSynced)
            .await
            .ok();

        Ok(())
    }
}

/// Fetches info for a coin's puzzle and inserts it into the database.
#[instrument(skip(peer, db, command_sender))]
async fn fetch_puzzle(
    peer: &Peer,
    db: &Database,
    genesis_challenge: Bytes32,
    coin_state: CoinState,
    command_sender: mpsc::Sender<SyncCommand>,
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
        ChildKind::from_parent(
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
        ChildKind::Launcher => {}
        ChildKind::Cat {
            asset_id,
            lineage_proof,
            p2_puzzle_hash,
        } => {
            tx.sync_coin(coin_id, Some(p2_puzzle_hash)).await?;
            tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                .await?;

            command_sender
                .send(SyncCommand::SubscribeCoin { coin_id })
                .await
                .ok();
        }
        ChildKind::Did {
            lineage_proof,
            info,
        } => {
            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;
            tx.insert_new_did(info.launcher_id, None, true).await?;
            tx.insert_did_coin(coin_id, lineage_proof, info).await?;

            command_sender
                .send(SyncCommand::SubscribeCoin { coin_id })
                .await
                .ok();
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
        } => {
            let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
            let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
            let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;
            tx.insert_new_nft(info.launcher_id, true).await?;
            tx.insert_nft_coin(
                coin_id,
                lineage_proof,
                info,
                data_hash,
                metadata_hash,
                license_hash,
            )
            .await?;

            if let Some(metadata) = metadata {
                if let Some(hash) = data_hash {
                    for uri in metadata.data_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }

                if let Some(hash) = metadata_hash {
                    for uri in metadata.metadata_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }

                if let Some(hash) = license_hash {
                    for uri in metadata.license_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }
            }

            command_sender
                .send(SyncCommand::SubscribeCoin { coin_id })
                .await
                .ok();
        }
        ChildKind::Unknown { hint } => {
            tx.sync_coin(coin_id, hint).await?;
            tx.insert_unknown_coin(coin_id).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}
