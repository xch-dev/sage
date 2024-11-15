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

use crate::{safely_remove_transaction, PeerState, SyncEvent, WalletError, WalletPeer};

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

            let mut mempool = false;
            let mut resolved = false;

            for peer in peers {
                match submit_transaction(peer, spend_bundle.clone(), self.genesis_challenge).await?
                {
                    Status::Pending => {
                        mempool = true;
                    }
                    Status::Success => {
                        info!(
                            "Transaction {transaction_id} confirmed, removing and confirming coins"
                        );

                        let mut tx = self.db.tx().await?;
                        tx.confirm_coins(transaction_id).await?;
                        safely_remove_transaction(&mut tx, transaction_id).await?;
                        tx.commit().await?;

                        resolved = true;
                        break;
                    }
                    Status::Failed => {
                        info!("Transaction {transaction_id} failed");

                        let mut tx = self.db.tx().await?;
                        safely_remove_transaction(&mut tx, transaction_id).await?;
                        tx.commit().await?;

                        resolved = true;
                        break;
                    }
                    Status::Unknown => {}
                }
            }

            if resolved {
                continue;
            }

            if mempool {
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
        }

        self.sync_sender.send(SyncEvent::Transaction).await.ok();

        Ok(())
    }
}

enum Status {
    Pending,
    Success,
    Failed,
    Unknown,
}

async fn submit_transaction(
    peer: WalletPeer,
    spend_bundle: SpendBundle,
    genesis_challenge: Bytes32,
) -> Result<Status, WalletError> {
    let ack = match timeout(
        Duration::from_secs(3),
        peer.send_transaction(spend_bundle.clone()),
    )
    .await
    {
        Ok(Ok(ack)) => ack,
        Err(_timeout) => {
            warn!("Send transaction timed out for {}", peer.socket_addr());
            return Ok(Status::Unknown);
        }
        Ok(Err(err)) => {
            warn!(
                "Send transaction failed for {}: {}",
                peer.socket_addr(),
                err
            );
            return Ok(Status::Unknown);
        }
    };

    info!(
        "Transaction sent to {} with ack {:?}",
        peer.socket_addr(),
        ack
    );

    if ack.error.is_none() {
        return Ok(Status::Pending);
    };

    let coin_ids: Vec<Bytes32> = spend_bundle
        .coin_spends
        .iter()
        .map(|cs| cs.coin.coin_id())
        .collect();

    let coin_states = match timeout(
        Duration::from_secs(2),
        peer.fetch_coins(coin_ids.clone(), genesis_challenge),
    )
    .await
    {
        Ok(Ok(coin_states)) => coin_states,
        Err(_timeout) => {
            warn!("Coin lookup timed out for {}", peer.socket_addr());
            return Ok(Status::Unknown);
        }
        Ok(Err(err)) => {
            warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
            return Ok(Status::Unknown);
        }
    };

    if coin_states.iter().all(|cs| cs.spent_height.is_some())
        && coin_ids
            .into_iter()
            .all(|coin_id| coin_states.iter().any(|cs| cs.coin.coin_id() == coin_id))
    {
        return Ok(Status::Success);
    } else if coin_states.iter().any(|cs| cs.spent_height.is_some()) {
        return Ok(Status::Failed);
    }

    Ok(Status::Failed)
}
