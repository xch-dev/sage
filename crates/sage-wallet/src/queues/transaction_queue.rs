use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::protocol::{Bytes32, SpendBundle};
use sage_database::Database;
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{info, warn};

use crate::{
    safely_remove_transaction, submit_to_peers, PeerState, Status, SyncEvent, WalletError,
};

#[derive(Debug)]
pub struct TransactionQueue {
    db: Database,
    genesis_challenge: Bytes32,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl TransactionQueue {
    pub fn new(
        db: Database,
        genesis_challenge: Bytes32,
        state: Arc<Mutex<PeerState>>,
        sync_sender: mpsc::Sender<SyncEvent>,
    ) -> Self {
        Self {
            db,
            genesis_challenge,
            state,
            sync_sender,
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

        let timestamp: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .try_into()
            .expect("timestamp does not fit in i64");

        let mut spend_bundles = Vec::new();

        let rows = self.db.resubmittable_transactions(timestamp - 180).await?;

        if rows.is_empty() {
            return Ok(());
        }

        info!("Submitting the following transactions: {rows:?}");

        for (transaction_id, aggregated_signature) in rows {
            let coin_spends = self.db.coin_spends(transaction_id).await?;
            spend_bundles.push(SpendBundle::new(coin_spends, aggregated_signature));
        }

        for spend_bundle in spend_bundles {
            sleep(Duration::from_secs(1)).await;

            let transaction_id = spend_bundle.name();

            let peers = self.state.lock().await.peers();

            if peers.is_empty() {
                return Ok(());
            }

            match submit_to_peers(&peers, self.genesis_challenge, spend_bundle).await? {
                Status::Success => {
                    info!("Transaction {transaction_id} confirmed, removing and confirming coins");

                    let mut tx = self.db.tx().await?;
                    tx.confirm_coins(transaction_id).await?;
                    safely_remove_transaction(&mut tx, transaction_id).await?;
                    tx.commit().await?;

                    self.sync_sender
                        .send(SyncEvent::TransactionEnded {
                            transaction_id,
                            success: true,
                        })
                        .await
                        .ok();
                }
                Status::Pending => {
                    info!("Transaction inclusion in mempool successful, updating timestamp");

                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs()
                        .try_into()
                        .expect("timestamp exceeds i64");

                    self.db
                        .update_transaction_mempool_time(transaction_id, timestamp)
                        .await?;
                }
                Status::Failed(status, error) => {
                    info!(
                        "Transaction inclusion in mempool failed for all peers with status {status} and error {error:?}, removing transaction"
                    );

                    let mut tx = self.db.tx().await?;
                    safely_remove_transaction(&mut tx, transaction_id).await?;
                    tx.commit().await?;

                    self.sync_sender
                        .send(SyncEvent::TransactionEnded {
                            transaction_id,
                            success: false,
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
