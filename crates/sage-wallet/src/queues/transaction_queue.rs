use std::{sync::Arc, time::Duration};

use chia_wallet_sdk::prelude::*;
use sage_database::Database;
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
};
use tracing::{info, warn};

use crate::{PeerState, Status, SyncEvent, WalletError, submit_to_peers};

#[derive(Debug)]
pub struct TransactionQueue {
    db: Database,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
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

        let rows = self.db.mempool_items_to_submit(120, 3).await?;

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
