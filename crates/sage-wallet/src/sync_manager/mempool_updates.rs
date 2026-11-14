//! Mempool updates from CHIP-26. A full node with the `MEMPOOL_UPDATES` capability announces
//! the ids of mempool transactions that touch the wallet's subscriptions, both when they enter
//! the mempool and, for transactions already in it, when a subscription is added. The latter
//! means pending transactions are found again after a resync or on a fresh install.

use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex as StdMutex},
};

use chia_wallet_sdk::{
    chia::protocol::{MempoolRemoveReason, RemovedMempoolItem},
    prelude::*,
};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};

use crate::{
    SyncCommand, Transaction, Wallet, WalletError, insert_discovered_transaction,
    is_wallet_transaction, remove_failed_transaction,
};

use super::{PeerState, SyncEvent};

/// The `MEMPOOL_UPDATES` capability, sent in the handshake to opt into mempool updates.
pub(super) const MEMPOOL_UPDATES_CAPABILITY: u16 = 5;

/// The most announced transactions waiting to be fetched. A peer can announce any number of
/// them, so the rest are dropped rather than kept in memory. The node announces them again
/// when the wallet subscribes next.
pub(super) const MAX_PENDING_MEMPOOL_ITEMS: usize = 1000;

/// A transaction id announced by the peer at `ip`, which is where its spend bundle is fetched
/// from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct AnnouncedMempoolItem {
    pub ip: IpAddr,
    pub transaction_id: Bytes32,
}

/// Transactions that left the mempool for good while a discovery batch was running. A removal
/// can arrive after the batch fetched a transaction but before it was inserted, when there's
/// nothing in the database to remove yet, so the batch checks this after each insert.
///
/// The removal handler records the id before it looks in the database, and the batch looks
/// here after it commits, so whichever comes second sees the other.
pub(super) type FinalRemovals = Arc<StdMutex<HashSet<Bytes32>>>;

/// Fetches the announced transactions that the wallet doesn't know about yet, and inserts the
/// ones that involve its coins as pending transactions. A transaction that can't be fetched or
/// inserted is skipped, so it doesn't hold up the rest.
pub(super) async fn discover_mempool_items(
    wallet: Arc<Wallet>,
    state: Arc<Mutex<PeerState>>,
    items: Vec<AnnouncedMempoolItem>,
    final_removals: FinalRemovals,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
) -> Result<(), WalletError> {
    let mut known: HashSet<Bytes32> = wallet
        .db
        .mempool_items()
        .await?
        .into_iter()
        .map(|item| item.hash)
        .collect();

    for item in items {
        if !known.insert(item.transaction_id) {
            continue;
        }

        // Each one is shown as soon as it's inserted, since the rest can take a while to fetch.
        match discover_mempool_item(&wallet, &state, item, &final_removals, &command_sender).await {
            Ok(true) => {
                sync_sender.send(SyncEvent::CoinsUpdated).await.ok();
            }
            Ok(false) => {}
            Err(error) => warn!(
                "Failed to discover mempool item {} from peer {}: {error}",
                item.transaction_id, item.ip
            ),
        }
    }

    Ok(())
}

/// Returns whether the transaction was inserted.
async fn discover_mempool_item(
    wallet: &Wallet,
    state: &Mutex<PeerState>,
    AnnouncedMempoolItem { ip, transaction_id }: AnnouncedMempoolItem,
    final_removals: &FinalRemovals,
    command_sender: &mpsc::Sender<SyncCommand>,
) -> Result<bool, WalletError> {
    // The node announces its own mempool, so only it is asked. If it disconnected, the
    // transaction is announced again by the peer the wallet subscribes with next.
    let Some(peer) = state.lock().await.peer(ip).map(|info| info.peer.clone()) else {
        debug!("Peer {ip} disconnected before mempool item {transaction_id} was fetched");
        return Ok(false);
    };

    // A node doesn't answer for a transaction that just left its mempool, so this times out.
    let spend_bundle = peer.fetch_mempool_transaction(transaction_id).await?;

    if spend_bundle.name() != transaction_id {
        warn!("Peer {ip} sent a different spend bundle for mempool item {transaction_id}");
        return Ok(false);
    }

    let transaction = Transaction::from_coin_spends(spend_bundle.coin_spends)?;

    // Checked first, since inserting fetches puzzle information for the outputs from the peer.
    if !is_wallet_transaction(&wallet.db, &transaction).await? {
        debug!("Mempool item {transaction_id} doesn't involve the wallet's coins");
        return Ok(false);
    }

    let Some(subscriptions) = insert_discovered_transaction(
        &wallet.db,
        &peer,
        wallet.genesis_challenge,
        transaction_id,
        transaction,
        spend_bundle.aggregated_signature,
    )
    .await?
    else {
        debug!("Mempool item {transaction_id} was confirmed while it was being fetched");
        return Ok(false);
    };

    let removed = final_removals
        .lock()
        .expect("final removals lock")
        .contains(&transaction_id);

    if removed {
        debug!("Mempool item {transaction_id} left the mempool while it was being fetched");
        remove_failed_transaction(&wallet.db, transaction_id).await?;
        return Ok(true);
    }

    info!("Discovered mempool item {transaction_id} from peer {ip}");

    if !subscriptions.is_empty() {
        command_sender
            .send(SyncCommand::SubscribeCoins {
                coin_ids: subscriptions,
            })
            .await
            .ok();
    }

    Ok(true)
}

/// Removes pending transactions that left the mempool for good. Ones that were included in a
/// block are confirmed by their coin states instead, and ones dropped because the mempool was
/// full can still be included, so they're left for the transaction queue to submit again.
pub(super) async fn remove_mempool_items(
    wallet: &Wallet,
    removed_items: Vec<RemovedMempoolItem>,
    final_removals: &FinalRemovals,
    sync_sender: &mpsc::Sender<SyncEvent>,
) -> Result<(), WalletError> {
    let removed: Vec<(Bytes32, SyncEvent)> = removed_items
        .into_iter()
        .filter_map(
            |RemovedMempoolItem {
                 transaction_id,
                 reason,
             }| {
                let event = match reason {
                    MempoolRemoveReason::BlockInclusion | MempoolRemoveReason::PoolFull => {
                        return None;
                    }
                    // Another spend of the same coins won, which the frontend treats as a double
                    // spend and doesn't report as a failure, so only the coins are refreshed.
                    MempoolRemoveReason::Conflict => SyncEvent::CoinsUpdated,
                    MempoolRemoveReason::Expired => SyncEvent::TransactionFailed {
                        transaction_id,
                        error: Some(
                            "The transaction expired before it was included in a block".to_string(),
                        ),
                    },
                };
                Some((transaction_id, event))
            },
        )
        .collect();

    if removed.is_empty() {
        return Ok(());
    }

    // Recorded before the database is read, so a discovery batch that inserts one of these
    // after the read removes it again itself.
    final_removals
        .lock()
        .expect("final removals lock")
        .extend(removed.iter().map(|(transaction_id, _)| *transaction_id));

    let known: HashSet<Bytes32> = wallet
        .db
        .mempool_items()
        .await?
        .into_iter()
        .map(|item| item.hash)
        .collect();

    for (transaction_id, event) in removed {
        if !known.contains(&transaction_id) {
            continue;
        }

        info!("Mempool item {transaction_id} left the mempool for good");

        remove_failed_transaction(&wallet.db, transaction_id).await?;

        sync_sender.send(event).await.ok();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chia_traits::Streamable;
    use chia_wallet_sdk::chia::protocol::{MempoolItemsAdded, Message, ProtocolMessageTypes};
    use test_log::test;
    use tokio::time::timeout;

    use crate::{SyncEvent, TestWallet, WalletPeer, mempool_peer, mempool_peer_serving};

    use super::*;

    const MEMPOOL_PEER_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1));

    /// Signs coin spends of `test`'s wallet without submitting them.
    async fn sign(test: &TestWallet, coin_spends: Vec<CoinSpend>) -> anyhow::Result<SpendBundle> {
        Ok(test
            .wallet
            .sign_transaction(
                SpendBundle::new(coin_spends, Signature::default()),
                &test.agg_sig,
                test.master_sk.clone(),
                false,
            )
            .await?)
    }

    /// A signed spend bundle sending `from`'s XCH to `to`, which isn't submitted.
    async fn unsubmitted_send(
        from: &TestWallet,
        to: Bytes32,
        amount: u64,
    ) -> anyhow::Result<SpendBundle> {
        let coin_spends = from
            .wallet
            .send_xch(vec![(to, amount)], 0, vec![], None)
            .await?;
        sign(from, coin_spends).await
    }

    /// Announces the transactions the way the peer's message handler forwards them.
    async fn announce(test: &TestWallet, transaction_ids: Vec<Bytes32>) -> anyhow::Result<()> {
        test.sender
            .send(SyncCommand::HandleMessage {
                ip: MEMPOOL_PEER_IP,
                message: Message {
                    msg_type: ProtocolMessageTypes::MempoolItemsAdded,
                    id: None,
                    data: MempoolItemsAdded::new(transaction_ids).to_bytes()?.into(),
                },
            })
            .await?;
        Ok(())
    }

    /// Runs a discovery batch against a peer that serves `spend_bundles`.
    struct Discovery {
        state: Arc<Mutex<PeerState>>,
        final_removals: FinalRemovals,
        events: mpsc::Receiver<SyncEvent>,
        event_sender: mpsc::Sender<SyncEvent>,
        commands: mpsc::Receiver<SyncCommand>,
        command_sender: mpsc::Sender<SyncCommand>,
    }

    impl Discovery {
        async fn new(peer: WalletPeer) -> anyhow::Result<Self> {
            let state = Arc::new(Mutex::new(PeerState::default()));
            state.lock().await.add_peer_at(MEMPOOL_PEER_IP, peer);

            let (event_sender, events) = mpsc::channel(10);
            let (command_sender, commands) = mpsc::channel(10);

            Ok(Self {
                state,
                final_removals: FinalRemovals::default(),
                events,
                event_sender,
                commands,
                command_sender,
            })
        }

        async fn serving(spend_bundles: Vec<SpendBundle>) -> anyhow::Result<Self> {
            Self::new(mempool_peer(spend_bundles).await?).await
        }

        async fn run(&self, test: &TestWallet, transaction_ids: &[Bytes32]) -> anyhow::Result<()> {
            let items = transaction_ids
                .iter()
                .map(|&transaction_id| AnnouncedMempoolItem {
                    ip: MEMPOOL_PEER_IP,
                    transaction_id,
                })
                .collect();

            timeout(
                Duration::from_mins(1),
                discover_mempool_items(
                    test.wallet.clone(),
                    self.state.clone(),
                    items,
                    self.final_removals.clone(),
                    self.event_sender.clone(),
                    self.command_sender.clone(),
                ),
            )
            .await??;

            Ok(())
        }

        async fn remove(
            &self,
            test: &TestWallet,
            transaction_id: Bytes32,
            reason: MempoolRemoveReason,
        ) -> anyhow::Result<()> {
            remove_mempool_items(
                &test.wallet,
                vec![RemovedMempoolItem::new(transaction_id, reason)],
                &self.final_removals,
                &self.event_sender,
            )
            .await?;
            Ok(())
        }
    }

    #[test(tokio::test)]
    async fn announced_transactions_are_discovered_after_a_resync() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;
        let first = unsubmitted_send(&test, test.puzzle_hash, 1000).await?;
        let first_id = first.name();

        // A resync starts from an empty database, which knows nothing about the transaction.
        test.resync().await?;
        assert!(test.wallet.db.mempool_items().await?.is_empty());

        let other = test.next(500).await?;
        let second = unsubmitted_send(&other, test.puzzle_hash, 500).await?;
        let second_id = second.name();

        test.state
            .lock()
            .await
            .add_peer_at(MEMPOOL_PEER_IP, mempool_peer(vec![first, second]).await?);

        // The node may announce a transaction more than once.
        announce(&test, vec![first_id, first_id]).await?;
        test.wait_for_coins().await;

        let items = test.wallet.db.mempool_items_with_coins().await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].hash, first_id);
        assert_eq!(items[0].spent.len(), 1);
        assert_eq!(items[0].created.len(), 1);

        // The coin it spends is pending, so it can't be spent again.
        assert!(test.wallet.db.selectable_xch_coins().await?.is_empty());

        // It's already in the node's mempool, so it isn't submitted again right away.
        assert!(
            test.wallet
                .db
                .mempool_items()
                .await?
                .iter()
                .all(|item| item.submitted_timestamp.is_some())
        );

        // A later announcement is fetched too, once the first batch has finished, and one
        // that's already known isn't inserted again.
        announce(&test, vec![first_id, second_id]).await?;
        test.wait_for_coins().await;

        let mut hashes: Vec<Bytes32> = test
            .wallet
            .db
            .mempool_items()
            .await?
            .into_iter()
            .map(|item| item.hash)
            .collect();
        hashes.sort();
        let mut expected = vec![first_id, second_id];
        expected.sort();
        assert_eq!(hashes, expected);

        Ok(())
    }

    #[test(tokio::test)]
    async fn incoming_payments_are_discovered_and_cleared_once_confirmed() -> anyhow::Result<()> {
        let mut test = TestWallet::new(0).await?;
        let other = test.next(1000).await?;

        let payment = unsubmitted_send(&other, test.puzzle_hash, 1000).await?;
        let payment_id = payment.name();

        let mut discovery = Discovery::serving(vec![payment.clone()]).await?;
        discovery.run(&test, &[payment_id]).await?;
        assert_eq!(discovery.events.try_recv()?, SyncEvent::CoinsUpdated);

        // Nothing of this wallet is spent, but the coin it's paid is shown as pending.
        let items = test.wallet.db.mempool_items_with_coins().await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].hash, payment_id);
        assert!(items[0].spent.is_empty());
        assert_eq!(items[0].created.len(), 1);
        assert_eq!(items[0].created[0].coin.amount, 1000);

        // The coin it creates is the only link to it, so its coin state confirms it.
        other.push_bundle(payment).await?;
        test.wait_for_coins().await;

        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn incoming_cats_are_discovered_and_subscribed_to() -> anyhow::Result<()> {
        let test = TestWallet::new(0).await?;
        let mut other = test.next(1000).await?;

        let (coin_spends, asset_id) = other.wallet.issue_cat(1000, 0, None).await?;
        other.transact(coin_spends).await?;
        other.wait_for_coins().await;

        // The CAT coin's puzzle hash isn't the wallet's, only its inner p2 puzzle hash is.
        let coin_spends = other
            .wallet
            .send_cat(
                asset_id,
                vec![(test.puzzle_hash, 1000)],
                0,
                true,
                vec![],
                None,
            )
            .await?;
        let payment = sign(&other, coin_spends).await?;
        let payment_id = payment.name();

        let mut discovery = Discovery::serving(vec![payment]).await?;
        discovery.run(&test, &[payment_id]).await?;

        let items = test.wallet.db.mempool_items_with_coins().await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].created.len(), 1);
        let created = items[0].created[0].coin;
        assert_eq!(created.amount, 1000);

        // The CAT coin is subscribed to, so the wallet hears when it's created.
        let Some(SyncCommand::SubscribeCoins { coin_ids }) = discovery.commands.try_recv().ok()
        else {
            panic!("the CAT coin wasn't subscribed to");
        };
        assert_eq!(coin_ids, vec![created.coin_id()]);

        Ok(())
    }

    #[test(tokio::test)]
    async fn unrelated_mismatched_and_unanswered_transactions_are_ignored() -> anyhow::Result<()> {
        let test = TestWallet::new(1000).await?;
        let other = test.next(1000).await?;

        // Spends another wallet's coin and pays it back, so it doesn't involve this wallet.
        let unrelated = unsubmitted_send(&other, other.puzzle_hash, 1000).await?;
        let unrelated_id = unrelated.name();

        // The peer answers with a spend bundle of this wallet, but not the one it announced.
        let mismatched_id = Bytes32::new([7; 32]);

        // The peer doesn't have it, so it never answers.
        let unanswered_id = Bytes32::new([8; 32]);

        let mut discovery = Discovery::new(
            mempool_peer_serving(vec![
                (unrelated_id, unrelated),
                (
                    mismatched_id,
                    unsubmitted_send(&test, test.puzzle_hash, 1000).await?,
                ),
            ])
            .await?,
        )
        .await?;
        discovery
            .run(&test, &[unrelated_id, mismatched_id, unanswered_id])
            .await?;

        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert!(discovery.events.try_recv().is_err());

        Ok(())
    }

    #[test(tokio::test)]
    async fn a_transaction_confirmed_while_being_fetched_is_not_inserted() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;
        let spend_bundle = unsubmitted_send(&test, test.puzzle_hash, 1000).await?;
        let transaction_id = spend_bundle.name();

        // It's confirmed, and its coin states are synced, before the fetched copy is inserted.
        test.push_bundle(spend_bundle.clone()).await?;
        test.wait_for_coins().await;
        assert!(test.wallet.db.mempool_items().await?.is_empty());

        let mut discovery = Discovery::serving(vec![spend_bundle]).await?;
        discovery.run(&test, &[transaction_id]).await?;

        // Inserting it would have made the confirmed coin look pending again.
        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);
        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert!(discovery.events.try_recv().is_err());

        Ok(())
    }

    #[test(tokio::test)]
    async fn a_payment_confirmed_while_being_fetched_is_not_inserted() -> anyhow::Result<()> {
        let mut test = TestWallet::new(0).await?;
        let other = test.next(1000).await?;

        // None of the coins it spends are this wallet's, so only the coin it creates shows
        // that it's confirmed.
        let payment = unsubmitted_send(&other, test.puzzle_hash, 1000).await?;
        let payment_id = payment.name();
        other.push_bundle(payment.clone()).await?;
        test.wait_for_coins().await;
        assert_eq!(test.wallet.db.xch_balance().await?, 1000);

        let mut discovery = Discovery::serving(vec![payment]).await?;
        discovery.run(&test, &[payment_id]).await?;

        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);
        assert!(discovery.events.try_recv().is_err());

        Ok(())
    }

    #[test(tokio::test)]
    async fn a_transaction_removed_while_being_fetched_is_removed_again() -> anyhow::Result<()> {
        let test = TestWallet::new(1000).await?;
        let spend_bundle = unsubmitted_send(&test, test.puzzle_hash, 1000).await?;
        let transaction_id = spend_bundle.name();

        let discovery = Discovery::serving(vec![spend_bundle]).await?;

        // The removal arrives before it's in the database, so there's nothing to remove yet.
        discovery
            .remove(&test, transaction_id, MempoolRemoveReason::Expired)
            .await?;
        discovery.run(&test, &[transaction_id]).await?;

        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn only_final_removals_remove_transactions() -> anyhow::Result<()> {
        let test = TestWallet::new(1000).await?;
        let spend_bundle = unsubmitted_send(&test, test.puzzle_hash, 1000).await?;
        let transaction_id = spend_bundle.name();

        let mut discovery = Discovery::serving(vec![spend_bundle]).await?;

        for reason in [MempoolRemoveReason::Conflict, MempoolRemoveReason::Expired] {
            discovery.final_removals.lock().expect("lock").clear();
            discovery.run(&test, &[transaction_id]).await?;
            assert_eq!(discovery.events.try_recv()?, SyncEvent::CoinsUpdated);
            assert!(test.wallet.db.selectable_xch_coins().await?.is_empty());

            // It can still be confirmed, or included once there's room in the mempool.
            for reason in [
                MempoolRemoveReason::BlockInclusion,
                MempoolRemoveReason::PoolFull,
            ] {
                discovery.remove(&test, transaction_id, reason).await?;
            }
            assert_eq!(test.wallet.db.mempool_items().await?.len(), 1);
            assert!(discovery.events.try_recv().is_err());

            // A transaction the wallet doesn't have isn't reported.
            discovery
                .remove(&test, Bytes32::new([9; 32]), reason)
                .await?;
            assert!(discovery.events.try_recv().is_err());

            discovery.remove(&test, transaction_id, reason).await?;
            assert!(test.wallet.db.mempool_items().await?.is_empty());
            assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

            // Losing to a conflicting spend is a double spend, which isn't reported as a failure.
            let event = discovery.events.try_recv()?;
            if reason == MempoolRemoveReason::Expired {
                assert!(matches!(
                    event,
                    SyncEvent::TransactionFailed { transaction_id: id, error: Some(_) }
                        if id == transaction_id
                ));
            } else {
                assert_eq!(event, SyncEvent::CoinsUpdated);
            }
        }

        Ok(())
    }
}
