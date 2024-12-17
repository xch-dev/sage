use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::protocol::Bytes32;
use sage_database::{Database, OfferStatus};
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use tracing::warn;

use crate::{PeerState, SyncEvent, WalletError};

#[derive(Debug)]
pub struct OfferQueue {
    db: Database,
    genesis_challenge: Bytes32,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl OfferQueue {
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
        let peak_height = self.state.lock().await.peak().map_or(0, |peak| peak.0);

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let offers = self.db.active_offers().await?;

        for offer in offers {
            if offer
                .expiration_height
                .is_some_and(|height| height <= peak_height)
                || offer.expiration_timestamp.is_some_and(|ts| ts <= timestamp)
            {
                self.db
                    .update_offer_status(offer.offer_id, OfferStatus::Expired)
                    .await?;

                self.sync_sender
                    .send(SyncEvent::OfferUpdated {
                        offer_id: offer.offer_id,
                        status: OfferStatus::Expired,
                    })
                    .await
                    .ok();

                continue;
            }

            loop {
                let Some(peer) = self.state.lock().await.acquire_peer() else {
                    return Ok(());
                };

                let coin_ids = self.db.offer_coin_ids(offer.offer_id).await?;

                let coin_states = match timeout(
                    Duration::from_secs(5),
                    peer.fetch_coins(coin_ids.clone(), self.genesis_challenge),
                )
                .await
                {
                    Ok(Ok(coin_states)) => coin_states,
                    Err(_timeout) => {
                        warn!("Coin lookup timed out for {}", peer.socket_addr());
                        self.state.lock().await.ban(
                            peer.socket_addr().ip(),
                            Duration::from_secs(300),
                            "coin lookup timeout",
                        );
                        continue;
                    }
                    Ok(Err(err)) => {
                        warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
                        self.state.lock().await.ban(
                            peer.socket_addr().ip(),
                            Duration::from_secs(300),
                            "coin lookup failed",
                        );
                        continue;
                    }
                };

                if coin_states.iter().all(|cs| cs.spent_height.is_some())
                    && coin_ids
                        .into_iter()
                        .all(|coin_id| coin_states.iter().any(|cs| cs.coin.coin_id() == coin_id))
                {
                    self.db
                        .update_offer_status(offer.offer_id, OfferStatus::Completed)
                        .await?;

                    self.sync_sender
                        .send(SyncEvent::OfferUpdated {
                            offer_id: offer.offer_id,
                            status: OfferStatus::Completed,
                        })
                        .await
                        .ok();
                } else if coin_states.iter().any(|cs| cs.spent_height.is_some()) {
                    self.db
                        .update_offer_status(offer.offer_id, OfferStatus::Cancelled)
                        .await?;

                    self.sync_sender
                        .send(SyncEvent::OfferUpdated {
                            offer_id: offer.offer_id,
                            status: OfferStatus::Cancelled,
                        })
                        .await
                        .ok();
                }

                break;
            }
        }

        Ok(())
    }
}
