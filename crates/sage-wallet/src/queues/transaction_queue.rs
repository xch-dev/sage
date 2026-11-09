use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use chia_wallet_sdk::prelude::*;
use sage_database::Database;
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
};
use tracing::{info, warn};

use crate::{PeerState, Status, SyncEvent, WalletError, submit_to_peers};

const RESUBMIT_INTERVAL: Duration = Duration::from_mins(2);
const BATCH_SIZE: usize = 3;

#[derive(Debug)]
pub struct TransactionQueue {
    db: Database,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
    /// When each item was last submitted, whatever the outcome. Only `Status::Pending`
    /// updates the database timestamp, so without this an item that keeps coming back
    /// as `Status::Unknown` would be picked first every batch and starve the rest.
    last_attempts: HashMap<Bytes32, Instant>,
}

impl TransactionQueue {
    pub fn new(
        db: Database,
        state: Arc<Mutex<PeerState>>,
        sync_sender: mpsc::Sender<SyncEvent>,
    ) -> Self {
        Self {
            db,
            state,
            sync_sender,
            last_attempts: HashMap::new(),
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

        let mut spend_bundles = Vec::new();

        let now = Instant::now();

        self.last_attempts
            .retain(|_, attempted_at| now.duration_since(*attempted_at) < RESUBMIT_INTERVAL);

        let rows: Vec<_> = self
            .db
            .mempool_items_to_submit(RESUBMIT_INTERVAL.as_secs().cast_signed())
            .await?
            .into_iter()
            .filter(|row| !self.last_attempts.contains_key(&row.hash))
            .take(BATCH_SIZE)
            .collect();

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let coin_spends = self.db.mempool_coin_spends(row.hash).await?;
            spend_bundles.push(SpendBundle::new(coin_spends, row.aggregated_signature));
        }

        for spend_bundle in spend_bundles {
            sleep(Duration::from_secs(1)).await;

            let peers = self.state.lock().await.peers();

            if peers.is_empty() {
                return Ok(());
            }

            let transaction_id = spend_bundle.name();

            info!(
                "Submitting transaction with id {transaction_id}: {:?}",
                spend_bundle
                    .coin_spends
                    .iter()
                    .map(|cs| cs.coin.coin_id())
                    .collect::<Vec<_>>()
            );

            self.last_attempts.insert(transaction_id, Instant::now());

            match submit_to_peers(&peers, spend_bundle).await? {
                Status::Pending => {
                    info!("Transaction inclusion in mempool successful, updating timestamp");

                    self.db.update_mempool_item_time(transaction_id).await?;

                    self.sync_sender
                        .send(SyncEvent::TransactionUpdated { transaction_id })
                        .await
                        .ok();
                }
                Status::Failed(status, error) => {
                    info!(
                        "Transaction inclusion in mempool failed for all peers with status {status} and error {error:?}, removing transaction"
                    );

                    let mut tx = self.db.tx().await?;

                    tx.set_transaction_children_unsynced(transaction_id).await?;
                    tx.remove_mempool_item(transaction_id).await?;

                    tx.commit().await?;

                    self.sync_sender
                        .send(SyncEvent::TransactionFailed {
                            transaction_id,
                            error,
                        })
                        .await
                        .ok();
                }
                Status::Unknown => {
                    warn!("Transaction inclusion in mempool unknown, retrying later");
                }
            }
        }

        Ok(())
    }
}
