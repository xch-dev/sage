use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::protocol::{Bytes32, SpendBundle};
use sage_database::Database;
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use tracing::{info, warn};

use crate::{safely_remove_transaction, PeerState, SyncEvent, WalletError};

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

            info!(
                "Broadcasting transaction id {}: {:?}",
                transaction_id, spend_bundle
            );

            let mut tx_confirmed = false;
            let mut tx_removed = false;

            for peer in peers.clone() {
                let coin_states = match timeout(
                    Duration::from_secs(2),
                    peer.fetch_coins(
                        spend_bundle
                            .coin_spends
                            .iter()
                            .map(|cs| cs.coin.coin_id())
                            .collect(),
                        self.genesis_challenge,
                    ),
                )
                .await
                {
                    Ok(Ok(coin_states)) => coin_states,
                    Err(_timeout) => {
                        warn!("Coin lookup timed out for {}", peer.socket_addr());
                        continue;
                    }
                    Ok(Err(err)) => {
                        warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
                        continue;
                    }
                };

                if coin_states.iter().all(|cs| cs.spent_height.is_some()) {
                    tx_confirmed = true;
                    break;
                } else if coin_states.iter().any(|cs| cs.spent_height.is_some()) {
                    tx_removed = true;
                    break;
                }
            }

            if tx_confirmed {
                info!("Transaction {transaction_id} confirmed, removing and confirming coins");

                let mut tx = self.db.tx().await?;
                tx.confirm_coins(transaction_id).await?;
                safely_remove_transaction(&mut tx, transaction_id).await?;
                tx.commit().await?;

                continue;
            } else if tx_removed {
                info!("Transaction {transaction_id} failed");

                let mut tx = self.db.tx().await?;
                safely_remove_transaction(&mut tx, transaction_id).await?;
                tx.commit().await?;

                continue;
            }

            let mut successful = false;

            for peer in peers {
                let ack = match timeout(
                    Duration::from_secs(3),
                    peer.send_transaction(spend_bundle.clone()),
                )
                .await
                {
                    Ok(Ok(ack)) => ack,
                    Err(_timeout) => {
                        warn!("Transaction timed out for {}", peer.socket_addr());
                        continue;
                    }
                    Ok(Err(err)) => {
                        warn!("Transaction failed for {}: {}", peer.socket_addr(), err);
                        continue;
                    }
                };

                successful = true;

                info!(
                    "Transaction sent to {} with ack {:?}",
                    peer.socket_addr(),
                    ack
                );
            }

            if successful {
                info!("Transaction inclusion in mempool successful, updating timestamp");

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs()
                    .try_into()
                    .expect("timestamp exceeds i64");

                self.db
                    .update_transaction_mempool_time(transaction_id, timestamp)
                    .await?;
            } else {
                info!("Transaction {transaction_id} failed");
                let mut tx = self.db.tx().await?;
                safely_remove_transaction(&mut tx, transaction_id).await?;
                tx.commit().await?;
            }
        }

        self.sync_sender.send(SyncEvent::CoinState).await.ok();

        Ok(())
    }
}
