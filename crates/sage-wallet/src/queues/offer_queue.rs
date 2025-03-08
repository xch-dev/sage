use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::Offer;
use clvmr::Allocator;
use sage_database::{Database, OfferStatus};
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use tracing::warn;

use crate::{parse_locked_coins, PeerState, SyncEvent, WalletError};

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
        if self.state.lock().await.peer_count() == 0 {
            return Ok(());
        }

        let offers = self.db.active_offers().await?;

        let mut settlement_coin_ids = HashMap::new();
        let mut input_coin_ids = HashMap::new();

        for offer in &offers {
            let mut allocator = Allocator::new();
            let parsed = Offer::decode(&offer.encoded_offer)?.parse(&mut allocator)?;
            let (locked, inputs) = parse_locked_coins(&mut allocator, &parsed)?;

            for coin in locked.coins() {
                settlement_coin_ids
                    .entry(coin.coin_id())
                    .or_insert(HashSet::new())
                    .insert(offer.offer_id);
            }

            for coin_id in inputs {
                input_coin_ids
                    .entry(coin_id)
                    .or_insert(HashSet::new())
                    .insert(offer.offer_id);
            }
        }

        let Some(peer) = self.state.lock().await.acquire_peer() else {
            return Ok(());
        };

        let coin_states = match timeout(
            Duration::from_secs(10),
            peer.fetch_coins(
                settlement_coin_ids
                    .keys()
                    .copied()
                    .chain(input_coin_ids.keys().copied())
                    .collect(),
                self.genesis_challenge,
            ),
        )
        .await
        {
            Ok(Ok(coin_states)) => coin_states,
            Ok(Err(err)) => {
                warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
                self.state.lock().await.ban(
                    peer.socket_addr().ip(),
                    Duration::from_secs(300),
                    "coin lookup failed",
                );
                return Ok(());
            }
            Err(_) => {
                warn!("Coin lookup timed out for {}", peer.socket_addr());
                self.state.lock().await.ban(
                    peer.socket_addr().ip(),
                    Duration::from_secs(300),
                    "coin lookup timeout",
                );
                return Ok(());
            }
        };

        let mut new_offer_statuses = HashMap::new();

        for coin_state in coin_states {
            if let Some(offer_ids) = settlement_coin_ids.get(&coin_state.coin.coin_id()) {
                for &offer_id in offer_ids {
                    new_offer_statuses.insert(offer_id, OfferStatus::Completed);
                }
            }

            if coin_state.spent_height.is_none() {
                continue;
            }

            if let Some(offer_ids) = input_coin_ids.get(&coin_state.coin.coin_id()) {
                for &offer_id in offer_ids {
                    new_offer_statuses
                        .entry(offer_id)
                        .or_insert(OfferStatus::Cancelled);
                }
            }
        }

        let peak_height = self.state.lock().await.peak().map_or(0, |peak| peak.0);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        for offer in offers {
            if offer
                .expiration_height
                .is_some_and(|height| height <= peak_height)
                || offer.expiration_timestamp.is_some_and(|ts| ts <= timestamp)
            {
                new_offer_statuses
                    .entry(offer.offer_id)
                    .or_insert(OfferStatus::Expired);
            }
        }

        for (offer_id, status) in new_offer_statuses {
            self.db.update_offer_status(offer_id, status).await?;

            self.sync_sender
                .send(SyncEvent::OfferUpdated { offer_id, status })
                .await
                .ok();
        }

        Ok(())
    }
}
