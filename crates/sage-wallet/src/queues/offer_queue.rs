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
        let mut pending_offer_coins = HashMap::new();

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

            let input_coins = offer
                .cancellable_coin_spends()?
                .into_iter()
                .map(|coin_spend| coin_spend.coin)
                .collect::<Vec<_>>();

            for coin in &input_coins {
                input_coin_ids
                    .entry(coin.coin_id())
                    .or_insert(HashSet::new())
                    .insert(row.offer_id);
            }

            if row.status == OfferStatus::Pending {
                pending_offer_coins.insert(row.offer_id, input_coins);
            }
        }

        // The parents of a pending offer's coins are the coins being split, which show whether
        // the split can still happen.
        let split_coin_ids = pending_offer_coins
            .values()
            .flatten()
            .map(|coin| coin.parent_coin_info)
            .collect::<HashSet<_>>();

        let Some(peer) = self.state.lock().await.acquire_peer() else {
            return Ok(());
        };

        let coin_states = match peer
            .fetch_coins(
                settlement_coin_ids
                    .keys()
                    .chain(input_coin_ids.keys())
                    .chain(split_coin_ids.iter())
                    .copied()
                    .collect::<HashSet<_>>()
                    .into_iter()
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
        let mut chain_coin_states = HashMap::new();

        for coin_state in coin_states {
            chain_coin_states.insert(coin_state.coin.coin_id(), coin_state);

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

        for (offer_id, coins) in pending_offer_coins {
            if new_offer_statuses.contains_key(&offer_id) {
                continue;
            }

            if let Some(status) = self
                .pending_offer_status(offer_id, &coins, &chain_coin_states)
                .await?
            {
                new_offer_statuses.insert(offer_id, status);
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
    /// Checks whether the coins a pending offer spends have been created, so it can be shared,
    /// or whether the transaction splitting them can no longer happen.
    async fn pending_offer_status(
        &self,
        offer_id: Bytes32,
        coins: &[Coin],
        chain_coin_states: &HashMap<Bytes32, CoinState>,
    ) -> Result<Option<OfferStatus>, WalletError> {
        let mut is_created = true;
        let mut tx = self.db.tx().await?;

        for coin in coins {
            let coin_id = coin.coin_id();

            if let Some(heights) = tx.coin_heights(coin_id).await? {
                // If the wallet saw the coins being split spent before it saw the split coins
                // created, it forgot the split coins until they synced again, which also dropped
                // their reservation for the offer. Reserving them again on every check keeps
                // them from being selected for long in that case.
                tx.insert_offered_coin(offer_id, coin_id).await?;

                is_created &= heights.created_height.is_some();
                continue;
            }

            is_created = false;

            // The coin isn't known, so it either failed to be created, or it was created and
            // the wallet is still syncing it.
            let Some(parent_heights) = tx.coin_heights(coin.parent_coin_info).await? else {
                // The action system makes split coins before the change, so they come straight
                // from the coins being split, and only the change is made from an intermediate
                // coin if it would be identical to one. In case a split coin is ever made from
                // one instead, the wallet forgets it too if the split fails, so if no peer has
                // seen it either, it was never created.
                if !chain_coin_states.contains_key(&coin.parent_coin_info) {
                    tx.commit().await?;
                    return Ok(Some(OfferStatus::Cancelled));
                }

                continue;
            };

            // When the split is confirmed or fails, the wallet stops tracking the coins it would
            // create. If the coin being split was never spent, the split failed. The wallet may
            // not have synced the spend yet, so a peer has to agree it's unspent.
            let parent_unspent_on_chain = chain_coin_states
                .get(&coin.parent_coin_info)
                .is_some_and(|state| state.spent_height.is_none());

            if parent_heights.spent_height.is_none() {
                if parent_unspent_on_chain {
                    tx.commit().await?;
                    return Ok(Some(OfferStatus::Cancelled));
                }

                continue;
            }

            // Otherwise it was spent, which only means the split failed if it was spent without
            // creating the coin. A peer which has seen the spend has seen the coin created too.
            let parent_spent = chain_coin_states
                .get(&coin.parent_coin_info)
                .is_some_and(|state| state.spent_height.is_some());
            let coin_created = chain_coin_states
                .get(&coin_id)
                .is_some_and(|state| state.created_height.is_some());

            if parent_spent && !coin_created {
                tx.commit().await?;
                return Ok(Some(OfferStatus::Cancelled));
            }
        }

        tx.commit().await?;

        Ok(is_created.then_some(OfferStatus::Active))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use chia_wallet_sdk::driver::encode_offer;
    use sage_database::OfferRow;
    use test_log::test;

    use crate::{Offered, PreSplitOffer, Requested, TestWallet};

    use super::*;

    /// Makes an offer of 300 mojos from a wallet with a single 1000 mojo coin, so the coin has
    /// to be split.
    async fn pre_split_offer(wallet: &TestWallet) -> anyhow::Result<(SpendBundle, Vec<CoinSpend>)> {
        pre_split_offer_of(wallet, 300).await
    }

    async fn pre_split_offer_of(
        wallet: &TestWallet,
        xch: u64,
    ) -> anyhow::Result<(SpendBundle, Vec<CoinSpend>)> {
        let PreSplitOffer {
            offer,
            split_coin_spends,
        } = wallet
            .wallet
            .make_pre_split_offer(
                Offered {
                    xch,
                    ..Default::default()
                },
                Requested {
                    xch: 1,
                    ..Default::default()
                },
                None,
                0,
            )
            .await?;

        Ok((offer, split_coin_spends.expect("the coin should be split")))
    }

    fn offer_coins(offer: &SpendBundle) -> anyhow::Result<Vec<Coin>> {
        let mut allocator = Allocator::new();

        Ok(Offer::from_spend_bundle(&mut allocator, offer)?
            .cancellable_coin_spends()?
            .into_iter()
            .map(|coin_spend| coin_spend.coin)
            .collect())
    }

    /// Saves an offer as pending, like making a pre-split offer does. Its coins are only
    /// reserved for it if `reserve` is set and the wallet knows about them.
    async fn insert_pending_offer(
        wallet: &TestWallet,
        offer: &SpendBundle,
        reserve: bool,
    ) -> anyhow::Result<Bytes32> {
        let offer_id = offer.name();
        let mut tx = wallet.wallet.db.tx().await?;

        tx.insert_offer(OfferRow {
            offer_id,
            encoded_offer: encode_offer(offer)?,
            expiration_height: None,
            expiration_timestamp: None,
            fee: 0,
            status: OfferStatus::Pending,
            inserted_timestamp: 0,
        })
        .await?;

        if reserve {
            for coin in offer_coins(offer)? {
                tx.insert_offered_coin(offer_id, coin.coin_id()).await?;
            }
        }

        tx.commit().await?;

        Ok(offer_id)
    }

    async fn sign(wallet: &TestWallet, coin_spends: Vec<CoinSpend>) -> anyhow::Result<SpendBundle> {
        Ok(wallet
            .wallet
            .sign_transaction(
                SpendBundle::new(coin_spends, Signature::default()),
                &wallet.agg_sig,
                wallet.master_sk.clone(),
                false,
            )
            .await?)
    }

    /// Waits for the offer's status to change, and returns the new status.
    async fn next_offer_status(wallet: &mut TestWallet, offer_id: Bytes32) -> OfferStatus {
        let status = Cell::new(None);

        wallet
            .consume_until(|event| {
                if let SyncEvent::OfferUpdated {
                    offer_id: updated_id,
                    status: updated_status,
                } = event
                    && updated_id == offer_id
                {
                    status.set(Some(updated_status));
                    return true;
                }

                false
            })
            .await;

        status.get().expect("status should be set")
    }

    async fn selectable_amounts(wallet: &TestWallet) -> anyhow::Result<Vec<u64>> {
        let mut amounts = wallet
            .wallet
            .db
            .selectable_xch_coins()
            .await?
            .into_iter()
            .map(|coin| coin.amount)
            .collect::<Vec<_>>();

        amounts.sort_unstable();

        Ok(amounts)
    }

    #[test(tokio::test)]
    async fn test_pending_offer_cancelled_when_split_fails() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;

        let (offer, split_coin_spends) = pre_split_offer(&alice).await?;

        // An unsigned split is rejected by the mempool, like a split that fails for any reason.
        alice
            .push_bundle(SpendBundle::new(split_coin_spends, Signature::default()))
            .await?;
        let offer_id = insert_pending_offer(&alice, &offer, true).await?;

        assert_eq!(
            next_offer_status(&mut alice, offer_id).await,
            OfferStatus::Cancelled
        );

        // The coin that would have been split can be spent again.
        assert_eq!(selectable_amounts(&alice).await?, vec![1000]);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_pending_offer_cancelled_when_split_with_intermediate_coin_fails()
    -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;

        // The change is the same as the split coin, so it's made from an intermediate coin, which
        // the wallet also tracks and forgets when the split fails.
        let (offer, split_coin_spends) = pre_split_offer_of(&alice, 500).await?;
        assert_eq!(split_coin_spends.len(), 2);

        alice
            .push_bundle(SpendBundle::new(split_coin_spends, Signature::default()))
            .await?;
        let offer_id = insert_pending_offer(&alice, &offer, true).await?;

        assert_eq!(
            next_offer_status(&mut alice, offer_id).await,
            OfferStatus::Cancelled
        );

        assert_eq!(selectable_amounts(&alice).await?, vec![1000]);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_pending_offer_cancelled_when_split_coin_spent_elsewhere() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;

        let (offer, _split_coin_spends) = pre_split_offer(&alice).await?;

        // The coin the split would spend is spent by something else instead.
        let mut ctx = SpendContext::new();
        alice
            .wallet
            .spend(
                &mut ctx,
                vec![],
                &[Action::send(
                    Id::Xch,
                    Bytes32::new([1; 32]),
                    1000,
                    Memos::None,
                )],
            )
            .await?;
        alice.transact(ctx.take()).await?;
        alice.wait_for_coins().await;

        let offer_id = insert_pending_offer(&alice, &offer, true).await?;

        assert_eq!(
            next_offer_status(&mut alice, offer_id).await,
            OfferStatus::Cancelled
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_pending_offer_not_cancelled_before_split_syncs() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;

        let (offer, split_coin_spends) = pre_split_offer(&alice).await?;
        let split_coins = split_coin_spends
            .iter()
            .map(|coin_spend| coin_spend.coin)
            .collect::<Vec<_>>();
        let split = sign(&alice, split_coin_spends).await?;

        alice.push_bundle(split).await?;
        alice.wait_for_coins().await;

        // Put the wallet back in the state it's in when the split has confirmed, but the wallet
        // hasn't synced it yet: the coin being split looks unspent, and the wallet doesn't know
        // about the split coins.
        let mut tx = alice.wallet.db.tx().await?;

        for coin in offer_coins(&offer)? {
            tx.delete_coin(coin.coin_id()).await?;
        }

        for coin in split_coins {
            let created_height = tx
                .coin_heights(coin.coin_id())
                .await?
                .expect("the coin being split should be known")
                .created_height;

            tx.insert_coin(CoinState::new(coin, None, created_height))
                .await?;
        }

        tx.commit().await?;

        let offer_id = insert_pending_offer(&alice, &offer, true).await?;

        // The wallet never syncs the split again here, so the offer stays pending, rather than
        // being cancelled because the coin being split looks unspent.
        sleep(Duration::from_secs(1)).await;

        let row = alice
            .wallet
            .db
            .offer(offer_id)
            .await?
            .expect("the offer should exist");
        assert_eq!(row.status, OfferStatus::Pending);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_pending_offer_reserves_coins_again() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;

        let (offer, split_coin_spends) = pre_split_offer(&alice).await?;
        let split = sign(&alice, split_coin_spends).await?;

        alice.push_bundle(split).await?;
        alice.wait_for_coins().await;

        // The offer's coins aren't reserved, like when the wallet forgot them while syncing.
        let offer_id = insert_pending_offer(&alice, &offer, false).await?;

        assert_eq!(
            next_offer_status(&mut alice, offer_id).await,
            OfferStatus::Active
        );

        // Only the change can be spent, since the split coin is reserved for the offer.
        assert_eq!(selectable_amounts(&alice).await?, vec![700]);

        Ok(())
    }
}
