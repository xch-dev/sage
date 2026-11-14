//! Mempool updates from CHIP-26. A full node with the `MEMPOOL_UPDATES` capability announces
//! the ids of mempool transactions that touch the wallet's subscriptions, both when they enter
//! the mempool and, for transactions already in it, when a subscription is added. The latter
//! means pending transactions are found again after a resync or on a fresh install.

use std::{collections::HashSet, net::IpAddr, sync::Arc};

use chia_wallet_sdk::{
    chia::protocol::{MempoolRemoveReason, RemovedMempoolItem},
    prelude::*,
};
use sage_database::Database;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};

use crate::{SyncCommand, Transaction, Wallet, WalletError, insert_transaction};

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

/// Fetches the announced transactions that the wallet doesn't know about yet, and inserts the
/// ones that involve its coins as pending transactions.
pub(super) async fn discover_mempool_items(
    wallet: Arc<Wallet>,
    state: Arc<Mutex<PeerState>>,
    items: Vec<AnnouncedMempoolItem>,
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

    let mut inserted = 0;

    for AnnouncedMempoolItem { ip, transaction_id } in items {
        if !known.insert(transaction_id) {
            continue;
        }

        // The node announces its own mempool, so only it is asked. If it disconnected, the
        // transaction is announced again by the peer the wallet subscribes with next.
        let Some(peer) = state.lock().await.peer(ip).map(|info| info.peer.clone()) else {
            debug!("Peer {ip} disconnected before mempool item {transaction_id} was fetched");
            continue;
        };

        let spend_bundle = match peer.fetch_mempool_transaction(transaction_id).await {
            Ok(spend_bundle) => spend_bundle,
            Err(error) => {
                // It may have just left the mempool, in which case the node doesn't answer.
                debug!("Failed to fetch mempool item {transaction_id} from {ip}: {error}");
                continue;
            }
        };

        if spend_bundle.name() != transaction_id {
            warn!("Peer {ip} sent a different spend bundle for mempool item {transaction_id}");
            continue;
        }

        let transaction = match Transaction::from_coin_spends(spend_bundle.coin_spends) {
            Ok(transaction) => transaction,
            Err(error) => {
                debug!("Failed to parse mempool item {transaction_id}: {error}");
                continue;
            }
        };

        if !is_relevant(&wallet.db, &transaction).await? {
            debug!("Mempool item {transaction_id} doesn't involve the wallet's coins");
            continue;
        }

        info!("Discovered mempool item {transaction_id} from peer {ip}");

        let subscriptions = insert_transaction(
            &wallet.db,
            &peer,
            wallet.genesis_challenge,
            transaction_id,
            transaction,
            spend_bundle.aggregated_signature,
        )
        .await?;

        // It's already in the node's mempool, so there's no need to submit it right away. It's
        // still submitted again later like any other pending transaction, which also removes it
        // if it can no longer be included.
        wallet.db.update_mempool_item_time(transaction_id).await?;

        if !subscriptions.is_empty() {
            command_sender
                .send(SyncCommand::SubscribeCoins {
                    coin_ids: subscriptions,
                })
                .await
                .ok();
        }

        inserted += 1;
    }

    if inserted > 0 {
        sync_sender.send(SyncEvent::CoinsUpdated).await.ok();
    }

    Ok(())
}

/// Whether inserting the transaction would attach any of the wallet's coins to it, either as a
/// coin it spends or one it creates. This mirrors what [`insert_transaction`] attaches, so
/// transactions the wallet can't show aren't kept.
async fn is_relevant(db: &Database, transaction: &Transaction) -> Result<bool, WalletError> {
    let mut tx = db.tx().await?;

    let output_coin_ids: HashSet<Bytes32> = transaction
        .inputs
        .iter()
        .flat_map(|input| &input.outputs)
        .map(|output| output.coin.coin_id())
        .collect();

    for input in &transaction.inputs {
        let coin_id = input.coin_spend.coin.coin_id();

        if !output_coin_ids.contains(&coin_id) && tx.is_known_coin(coin_id).await? {
            return Ok(true);
        }

        for output in &input.outputs {
            if tx
                .is_custody_p2_puzzle_hash(output.coin.puzzle_hash)
                .await?
            {
                return Ok(true);
            }

            for p2_puzzle_hash in output.kind.custody_p2_puzzle_hashes() {
                if tx.is_custody_p2_puzzle_hash(p2_puzzle_hash).await? {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Removes pending transactions that left the mempool for good. Ones that were included in a
/// block are confirmed by their coin states instead, and ones dropped because the mempool was
/// full can still be included, so they're left for the transaction queue to submit again.
///
/// A conflict means another spend of the same coins won, which the frontend treats as a
/// double spend and doesn't report as a failure, so only the coins are refreshed.
pub(super) async fn remove_mempool_items(
    wallet: &Wallet,
    removed_items: Vec<RemovedMempoolItem>,
    sync_sender: &mpsc::Sender<SyncEvent>,
) -> Result<(), WalletError> {
    let known: HashSet<Bytes32> = wallet
        .db
        .mempool_items()
        .await?
        .into_iter()
        .map(|item| item.hash)
        .collect();

    for RemovedMempoolItem {
        transaction_id,
        reason,
    } in removed_items
    {
        if matches!(
            reason,
            MempoolRemoveReason::BlockInclusion | MempoolRemoveReason::PoolFull
        ) {
            continue;
        }

        if !known.contains(&transaction_id) {
            continue;
        }

        info!("Mempool item {transaction_id} was removed from the mempool: {reason:?}");

        let mut tx = wallet.db.tx().await?;
        tx.set_transaction_children_unsynced(transaction_id).await?;
        tx.remove_mempool_item(transaction_id).await?;
        tx.commit().await?;

        let event = if reason == MempoolRemoveReason::Expired {
            SyncEvent::TransactionFailed {
                transaction_id,
                error: Some(
                    "The transaction expired before it was included in a block".to_string(),
                ),
            }
        } else {
            SyncEvent::CoinsUpdated
        };

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

    use crate::{SyncEvent, TestWallet, mempool_peer, mempool_peer_serving};

    use super::*;

    const MEMPOOL_PEER_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1));

    /// A signed spend bundle sending the wallet's coin back to itself, which isn't submitted.
    async fn unsubmitted_send(test: &TestWallet) -> anyhow::Result<SpendBundle> {
        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

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

    #[test(tokio::test)]
    async fn announced_transactions_are_discovered_after_a_resync() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;
        let spend_bundle = unsubmitted_send(&test).await?;
        let transaction_id = spend_bundle.name();

        // A resync starts from an empty database, which knows nothing about the transaction.
        test.resync().await?;
        assert!(test.wallet.db.mempool_items().await?.is_empty());

        test.state
            .lock()
            .await
            .add_peer_at(MEMPOOL_PEER_IP, mempool_peer(vec![spend_bundle]).await?);

        announce(&test, vec![transaction_id]).await?;
        test.wait_for_coins().await;

        let items = test.wallet.db.mempool_items_with_coins().await?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].hash, transaction_id);
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

        Ok(())
    }

    #[test(tokio::test)]
    async fn unrelated_mismatched_and_unanswered_transactions_are_ignored() -> anyhow::Result<()> {
        let test = TestWallet::new(1000).await?;
        let other = test.next(1000).await?;

        // Spends another wallet's coin and pays it back, so it doesn't involve this wallet.
        let unrelated = unsubmitted_send(&other).await?;
        let unrelated_id = unrelated.name();

        // The peer answers with a spend bundle of this wallet, but not the one it announced.
        let mismatched_id = Bytes32::new([7; 32]);

        // The peer doesn't have it, so it never answers.
        let unanswered_id = Bytes32::new([8; 32]);

        let peer = mempool_peer_serving(vec![
            (unrelated_id, unrelated),
            (mismatched_id, unsubmitted_send(&test).await?),
        ])
        .await?;
        let state = Arc::new(Mutex::new(PeerState::default()));
        state.lock().await.add_peer_at(MEMPOOL_PEER_IP, peer);

        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (command_sender, _command_receiver) = mpsc::channel(10);

        let items = [unrelated_id, mismatched_id, unanswered_id]
            .into_iter()
            .map(|transaction_id| AnnouncedMempoolItem {
                ip: MEMPOOL_PEER_IP,
                transaction_id,
            })
            .collect();

        timeout(
            Duration::from_secs(30),
            discover_mempool_items(
                test.wallet.clone(),
                state,
                items,
                event_sender,
                command_sender,
            ),
        )
        .await??;

        assert!(test.wallet.db.mempool_items().await?.is_empty());
        assert!(event_receiver.try_recv().is_err());

        Ok(())
    }

    #[test(tokio::test)]
    async fn only_final_removals_remove_transactions() -> anyhow::Result<()> {
        let test = TestWallet::new(1000).await?;
        let spend_bundle = unsubmitted_send(&test).await?;
        let transaction_id = spend_bundle.name();

        let state = Arc::new(Mutex::new(PeerState::default()));
        state
            .lock()
            .await
            .add_peer_at(MEMPOOL_PEER_IP, mempool_peer(vec![spend_bundle]).await?);

        let (event_sender, mut event_receiver) = mpsc::channel(10);
        let (command_sender, _command_receiver) = mpsc::channel(10);

        let remove = async |reason| {
            remove_mempool_items(
                &test.wallet,
                vec![RemovedMempoolItem::new(transaction_id, reason)],
                &event_sender,
            )
            .await
        };

        for reason in [MempoolRemoveReason::Conflict, MempoolRemoveReason::Expired] {
            discover_mempool_items(
                test.wallet.clone(),
                state.clone(),
                vec![AnnouncedMempoolItem {
                    ip: MEMPOOL_PEER_IP,
                    transaction_id,
                }],
                event_sender.clone(),
                command_sender.clone(),
            )
            .await?;
            assert_eq!(event_receiver.try_recv()?, SyncEvent::CoinsUpdated);
            assert!(test.wallet.db.selectable_xch_coins().await?.is_empty());

            // It can still be confirmed, or included once there's room in the mempool.
            remove(MempoolRemoveReason::BlockInclusion).await?;
            remove(MempoolRemoveReason::PoolFull).await?;
            assert_eq!(test.wallet.db.mempool_items().await?.len(), 1);
            assert!(event_receiver.try_recv().is_err());

            remove(reason).await?;
            assert!(test.wallet.db.mempool_items().await?.is_empty());
            assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

            // Losing to a conflicting spend is a double spend, which isn't reported as a failure.
            let event = event_receiver.try_recv()?;
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
