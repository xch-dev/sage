use std::{collections::HashMap, sync::Arc, time::Duration};

use chia::protocol::{Bytes32, Coin};
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage_database::Database;
use tokio::{
    sync::{mpsc, Mutex},
    task::spawn_blocking,
    time::{sleep, timeout},
};
use tracing::{debug, warn};

use crate::{
    database::insert_puzzle, fetch_nft_did, ChildKind, PeerState, SyncCommand, SyncEvent,
    WalletError, WalletPeer,
};

#[derive(Debug)]
pub struct PuzzleQueue {
    db: Database,
    genesis_challenge: Bytes32,
    batch_size_per_peer: usize,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
}

impl PuzzleQueue {
    pub fn new(
        db: Database,
        genesis_challenge: Bytes32,
        batch_size_per_peer: usize,
        state: Arc<Mutex<PeerState>>,
        sync_sender: mpsc::Sender<SyncEvent>,
        command_sender: mpsc::Sender<SyncCommand>,
    ) -> Self {
        Self {
            db,
            genesis_challenge,
            batch_size_per_peer,
            state,
            sync_sender,
            command_sender,
        }
    }

    pub async fn start(mut self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&mut self) -> Result<(), WalletError> {
        let peers = self.state.lock().await.peers();

        if peers.is_empty() {
            return Ok(());
        }

        let coin_states = self
            .db
            .unsynced_coins(peers.len() * self.batch_size_per_peer)
            .await?;

        if coin_states.is_empty() {
            return Ok(());
        }

        debug!("Syncing a batch of {} coins", coin_states.len());

        let mut futures = FuturesUnordered::new();
        let mut remaining = coin_states.into_iter();

        for peer in peers {
            for _ in 0..self.batch_size_per_peer {
                let Some(coin_state) = remaining.next() else {
                    break;
                };
                let coin = coin_state.coin;

                let db = self.db.clone();
                let peer = peer.clone();
                let genesis_challenge = self.genesis_challenge;

                if let Some(p2_puzzle_id) = db.p2_puzzle_id(coin.puzzle_hash).await? {
                    let id = db.coin_id(coin.coin_id()).await?;
                    db.sync_coin(id, 0, p2_puzzle_id, None).await?;
                    warn!("Added missing XCH asset to {}", coin.coin_id());
                    continue;
                }

                futures.push(async move {
                    let result = fetch_puzzle(&peer, genesis_challenge, coin).await;
                    (peer.socket_addr(), coin_state, result)
                });
            }
        }

        let mut subscriptions = Vec::new();

        while let Some((addr, coin_state, result)) = futures.next().await {
            let coin_id = coin_state.coin.coin_id();

            match result {
                Ok((info, minter_did)) => {
                    let subscribe = info.subscribe();

                    let mut tx = self.db.tx().await?;

                    if insert_puzzle(&mut tx, coin_state, info, minter_did).await? && subscribe {
                        subscriptions.push(coin_id);
                    }

                    tx.commit().await?;
                }
                Err(error) => {
                    debug!(
                        "Failed to lookup puzzle of {} from peer {}: {}",
                        coin_id, addr, error
                    );

                    if matches!(
                        error,
                        WalletError::Elapsed(..)
                            | WalletError::PeerMisbehaved
                            | WalletError::Client(..)
                    ) {
                        self.state.lock().await.ban(
                            addr.ip(),
                            Duration::from_secs(300),
                            "failed puzzle lookup",
                        );
                    }
                }
            }
        }

        self.command_sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await
            .ok();

        self.sync_sender
            .send(SyncEvent::PuzzleBatchSynced)
            .await
            .ok();

        Ok(())
    }
}

/// Fetches info for a coin's puzzle and inserts it into the database.
async fn fetch_puzzle(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    coin: Coin,
) -> Result<(ChildKind, Option<Bytes32>), WalletError> {
    let parent_spend = timeout(
        Duration::from_secs(15),
        peer.fetch_coin_spend(coin.parent_coin_info, genesis_challenge),
    )
    .await??;

    let info = spawn_blocking(move || {
        ChildKind::from_parent(
            parent_spend.coin,
            &parent_spend.puzzle_reveal,
            &parent_spend.solution,
            coin,
        )
    })
    .await??;

    let minter_did = if let ChildKind::Nft { info, .. } = &info {
        fetch_nft_did(peer, genesis_challenge, info.launcher_id, &HashMap::new()).await?
    } else {
        None
    };

    Ok((info, minter_did))
}
