use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use chia_wallet_sdk::prelude::*;
use sage_database::{Database, MempoolItem};
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

        let due = self
            .db
            .mempool_items_to_submit(RESUBMIT_INTERVAL.as_secs().cast_signed())
            .await?;

        let rows = select_batch(due, &mut self.last_attempts, Instant::now());

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let coin_spends = self.db.mempool_coin_spends(row.hash).await?;
            spend_bundles.push((
                row.hash,
                SpendBundle::new(coin_spends, row.aggregated_signature),
            ));
        }

        for (transaction_id, spend_bundle) in spend_bundles {
            sleep(Duration::from_secs(1)).await;

            let peers = self.state.lock().await.peers();

            if peers.is_empty() {
                return Ok(());
            }

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

/// Picks up to [`BATCH_SIZE`] of the due items, in order, skipping any attempted within
/// [`RESUBMIT_INTERVAL`] of `now`. Attempts older than that are forgotten.
fn select_batch(
    due: Vec<MempoolItem>,
    last_attempts: &mut HashMap<Bytes32, Instant>,
    now: Instant,
) -> Vec<MempoolItem> {
    last_attempts.retain(|_, attempted_at| now.duration_since(*attempted_at) < RESUBMIT_INTERVAL);

    due.into_iter()
        .filter(|item| !last_attempts.contains_key(&item.hash))
        .take(BATCH_SIZE)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(byte: u8) -> MempoolItem {
        MempoolItem {
            hash: Bytes32::new([byte; 32]),
            aggregated_signature: Signature::default(),
            fee: 0,
            submitted_timestamp: None,
        }
    }

    fn hashes(items: &[MempoolItem]) -> Vec<Bytes32> {
        items.iter().map(|item| item.hash).collect()
    }

    #[test]
    fn select_batch_takes_at_most_a_batch_in_order() {
        let due: Vec<_> = (1..=5).map(item).collect();
        let batch = select_batch(due.clone(), &mut HashMap::new(), Instant::now());
        assert_eq!(hashes(&batch), hashes(&due[..BATCH_SIZE]));
    }

    #[test]
    fn select_batch_handles_no_due_items() {
        let mut last_attempts = HashMap::from([(item(1).hash, Instant::now())]);
        assert!(select_batch(Vec::new(), &mut last_attempts, Instant::now()).is_empty());
    }

    #[test]
    fn select_batch_skips_recent_attempts_until_the_interval_passes() {
        let start = Instant::now();
        let due: Vec<_> = (1..=5).map(item).collect();

        // Items 1 to 3 were just tried and came back unknown, so the next batch moves on
        // to the rest instead of trying them again.
        let mut last_attempts: HashMap<_, _> =
            due[..3].iter().map(|item| (item.hash, start)).collect();

        let batch = select_batch(due.clone(), &mut last_attempts, start);
        assert_eq!(hashes(&batch), hashes(&due[3..]));

        let just_before = start + RESUBMIT_INTERVAL.saturating_sub(Duration::from_secs(1));
        let batch = select_batch(due.clone(), &mut last_attempts, just_before);
        assert_eq!(hashes(&batch), hashes(&due[3..]));
        assert_eq!(last_attempts.len(), 3);

        // Once the interval has passed, the old attempts are forgotten and items 1 to 3
        // are due again.
        let batch = select_batch(due.clone(), &mut last_attempts, start + RESUBMIT_INTERVAL);
        assert_eq!(hashes(&batch), hashes(&due[..3]));
        assert!(last_attempts.is_empty());
    }
}
