use std::{sync::Arc, time::Duration};

use chia_wallet_sdk::prelude::*;
use futures_util::{stream::FuturesUnordered, StreamExt};
use sage_database::{Database, UnsyncedCoin};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{debug, info, warn};

use crate::{
    database::insert_puzzle, validate_wallet_coin, ChildKind, PeerState, PuzzleContext,
    SyncCommand, SyncEvent, WalletError, WalletPeer,
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

        let limit = peers.len() * self.batch_size_per_peer;

        let coin_states = self.db.unsynced_coins(limit).await?;

        if coin_states.is_empty() {
            return Ok(());
        }

        info!(
            "Syncing a batch of {} coins from {} peers",
            coin_states.len(),
            peers.len()
        );

        let mut futures = FuturesUnordered::new();
        let mut remaining = coin_states.into_iter();

        for peer in peers {
            for _ in 0..self.batch_size_per_peer {
                let Some(row) = remaining.next() else {
                    break;
                };

                let peer = peer.clone();
                let genesis_challenge = self.genesis_challenge;
                let is_custody_p2_puzzle_hash = self
                    .db
                    .is_custody_p2_puzzle_hash(row.coin_state.coin.puzzle_hash)
                    .await?;

                futures.push(async move {
                    let result =
                        fetch_puzzles(&peer, genesis_challenge, row, is_custody_p2_puzzle_hash)
                            .await;
                    (peer.socket_addr(), row, result)
                });
            }
        }

        let mut subscriptions = Vec::new();
        let mut send_events = false;

        while let Some((addr, root, synced_coins)) = futures.next().await {
            match synced_coins {
                Ok(synced_coins) => {
                    let mut tx = self.db.tx().await?;

                    if root.is_children_unsynced {
                        debug!(
                            "Children have been synced for coin {}",
                            root.coin_state.coin.coin_id()
                        );
                        tx.set_children_synced(root.coin_state.coin.coin_id())
                            .await?;
                        send_events = true;
                    }

                    for item in synced_coins {
                        let coin_id = item.coin_state.coin.coin_id();
                        let is_root = root.coin_state.coin.coin_id() == coin_id;

                        // We want to skip children that we already know about
                        if !is_root && tx.is_known_coin(coin_id).await? {
                            debug!("Skipping child coin {coin_id} because it is already known");
                            continue;
                        }

                        let is_custody_p2_puzzle_hash =
                            tx.is_custody_p2_puzzle_hash(coin_id).await?;

                        let Some(kind) = item.kind.filter(|_| !is_custody_p2_puzzle_hash) else {
                            warn!("Retroactively inserting XCH coin that should have already been synced: {coin_id}");

                            self.db
                                .update_coin(
                                    coin_id,
                                    Bytes32::default(),
                                    item.coin_state.coin.puzzle_hash,
                                )
                                .await?;
                            send_events = true;
                            continue;
                        };

                        // We don't want to insert child coins that we don't own.
                        let custody_p2_puzzle_hashes = kind.custody_p2_puzzle_hashes();

                        let mut is_relevant = false;

                        for custody_p2_puzzle_hash in custody_p2_puzzle_hashes {
                            if tx.is_custody_p2_puzzle_hash(custody_p2_puzzle_hash).await? {
                                is_relevant = true;
                                break;
                            }
                        }

                        if !is_relevant {
                            if is_root {
                                warn!("Deleting unexpected coin {coin_id} because it is not relevant to this wallet");
                                tx.delete_coin(coin_id).await?;
                                send_events = true;
                            } else {
                                debug!("Skipping coin {coin_id} because it is not relevant to this wallet");
                            }
                            continue;
                        }

                        if is_root {
                            debug!("Synced puzzle for coin {coin_id}");
                        } else {
                            debug!("Found relevant child coin {coin_id} which wasn't synced");
                        }

                        send_events = true;

                        if let Some(height) = item.coin_state.created_height {
                            tx.insert_height(height).await?;
                        }

                        if let Some(height) = item.coin_state.spent_height {
                            tx.insert_height(height).await?;
                        }

                        tx.insert_coin(item.coin_state).await?;

                        let is_inserted = validate_wallet_coin(&mut tx, coin_id, &kind).await?
                            && insert_puzzle(
                                &mut tx,
                                item.coin_state,
                                kind.clone(),
                                item.context.clone(),
                                None,
                            )
                            .await?;

                        if is_inserted {
                            if kind.subscribe() {
                                subscriptions.push(coin_id);
                            }

                            if let PuzzleContext::Option(context) = item.context {
                                subscriptions.push(context.underlying.coin.coin_id());
                            }
                        }
                    }

                    tx.commit().await?;
                }
                Err(error) => {
                    debug!(
                        "Failed to sync {} from peer {}: {}",
                        root.coin_state.coin.coin_id(),
                        addr,
                        error
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

        if send_events {
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
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SyncedCoin {
    coin_state: CoinState,
    kind: Option<ChildKind>,
    context: PuzzleContext,
}

async fn fetch_puzzles(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    unsynced_coin: UnsyncedCoin,
    is_custody_p2_puzzle_hash: bool,
) -> Result<Vec<SyncedCoin>, WalletError> {
    let coin = unsynced_coin.coin_state.coin;

    let mut synced_coins = Vec::new();

    if unsynced_coin.is_asset_unsynced {
        let parent_spend = if is_custody_p2_puzzle_hash {
            None
        } else {
            peer.fetch_optional_coin_spend(coin.parent_coin_info, genesis_challenge)
                .await?
        };

        if let Some(parent_spend) = parent_spend {
            let kind = ChildKind::from_parent(
                parent_spend.coin,
                &parent_spend.puzzle_reveal,
                &parent_spend.solution,
                coin,
            )?;

            let context = PuzzleContext::fetch(peer, genesis_challenge, &kind).await?;

            synced_coins.push(SyncedCoin {
                coin_state: unsynced_coin.coin_state,
                kind: Some(kind),
                context,
            });
        } else {
            synced_coins.push(SyncedCoin {
                coin_state: unsynced_coin.coin_state,
                kind: None,
                context: PuzzleContext::None,
            });
        }
    }

    if unsynced_coin.is_children_unsynced {
        if let Some(spent_height) = unsynced_coin.coin_state.spent_height {
            let (puzzle_reveal, solution) = peer
                .fetch_puzzle_solution(coin.coin_id(), spent_height)
                .await?;

            let children = ChildKind::parse_children(coin, &puzzle_reveal, &solution)?;

            let coin_states = peer
                .fetch_coins(
                    children.iter().map(|(child, _)| child.coin_id()).collect(),
                    genesis_challenge,
                )
                .await?;

            for (child_coin, kind) in children {
                let Some(&coin_state) = coin_states
                    .iter()
                    .find(|coin_state| coin_state.coin.coin_id() == child_coin.coin_id())
                else {
                    continue;
                };

                let context = PuzzleContext::fetch(peer, genesis_challenge, &kind).await?;

                synced_coins.push(SyncedCoin {
                    coin_state,
                    kind: Some(kind),
                    context,
                });
            }
        }
    }

    Ok(synced_coins)
}
