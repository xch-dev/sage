use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{driver::decode_offer, prelude::*};
use sage_database::{Database, OfferStatus};
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
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
        if self.state.lock().await.peer_count() == 0 {
            return Ok(());
        }

        // Pending offers spend coins that are still being split, and are checked the same way,
        // since they can expire or have their coins spent like any other offer.
        let mut offers = self.db.offers(Some(OfferStatus::Pending)).await?;
        offers.extend(self.db.offers(Some(OfferStatus::Active)).await?);

        if offers.is_empty() {
            return Ok(());
        }

        let mut settlement_coin_ids = HashMap::new();
        let mut input_coin_ids = HashMap::new();

        for row in &offers {
            let mut allocator = Allocator::new();

            let spend_bundle = decode_offer(&row.encoded_offer)?;
            let offer = Offer::from_spend_bundle(&mut allocator, &spend_bundle)?;

            for coin in offer.offered_coins().flatten() {
                settlement_coin_ids
                    .entry(coin.coin_id())
                    .or_insert(HashSet::new())
                    .insert(row.offer_id);
            }

            for coin_spend in offer.cancellable_coin_spends()? {
                input_coin_ids
                    .entry(coin_spend.coin.coin_id())
                    .or_insert(HashSet::new())
                    .insert(row.offer_id);
            }
        }

        let Some(peer) = self.state.lock().await.acquire_peer() else {
            return Ok(());
        };

        let coin_states = match peer
            .fetch_coins(
                settlement_coin_ids
                    .keys()
                    .copied()
                    .chain(input_coin_ids.keys().copied())
                    .collect(),
                self.genesis_challenge,
            )
            .await
        {
            Ok(coin_states) => coin_states,
            Err(err) => {
                warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
                self.state.lock().await.ban(
                    peer.socket_addr().ip(),
                    Duration::from_mins(5),
                    "coin lookup failed",
                );
                return Ok(());
            }
        };

        let mut new_offer_statuses = HashMap::new();
        let mut created_coin_ids = HashSet::new();

        for coin_state in &coin_states {
            if coin_state.created_height.is_some() {
                created_coin_ids.insert(coin_state.coin.coin_id());
            }
        }

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

        for offer in &offers {
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

        for offer in &offers {
            if offer.status != OfferStatus::Pending
                || new_offer_statuses.contains_key(&offer.offer_id)
            {
                continue;
            }

            let coin_ids = input_coin_ids
                .iter()
                .filter(|(_, offer_ids)| offer_ids.contains(&offer.offer_id))
                .map(|(&coin_id, _)| coin_id)
                .collect::<Vec<_>>();

            if coin_ids
                .iter()
                .all(|coin_id| created_coin_ids.contains(coin_id))
            {
                // If the wallet saw the split's inputs spent before its coins were created, it
                // forgot the coins until they synced again, which also dropped their reservation
                // for the offer, so they're reserved again.
                let mut tx = self.db.tx().await?;

                for &coin_id in &coin_ids {
                    tx.insert_offered_coin(offer.offer_id, coin_id).await?;
                }

                tx.commit().await?;

                new_offer_statuses.insert(offer.offer_id, OfferStatus::Active);
                continue;
            }

            // The wallet forgets the coins a transaction would have created if it fails, so if
            // one of them isn't on-chain or known anymore, the split failed and the offer can
            // never be taken.
            for coin_id in coin_ids {
                if !created_coin_ids.contains(&coin_id) && !self.db.is_known_coin(coin_id).await? {
                    new_offer_statuses.insert(offer.offer_id, OfferStatus::Cancelled);
                    break;
                }
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
