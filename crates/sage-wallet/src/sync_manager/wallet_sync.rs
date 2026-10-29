use std::{collections::HashSet, sync::Arc, time::Duration};

use chia_wallet_sdk::{chia::protocol::CoinStateFilters, prelude::*};
use sage_database::DatabaseTx;
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
};
use tracing::{info, warn};

use crate::{SyncCommand, Wallet, WalletError, WalletKind, WalletPeer};

use super::{PeerState, SyncEvent};

pub async fn sync_wallet(
    wallet: Arc<Wallet>,
    peer: WalletPeer,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
    delta_sync: bool,
) -> Result<(), WalletError> {
    info!("Starting sync against peer {}", peer.socket_addr());

    // Watched puzzle hashes without any coins may have been added since the last sync, such as
    // while the wallet was closed, so they're synced from the start rather than the last peak.
    // One that never had coins is cheap to check again.
    let unsynced: HashSet<Bytes32> = wallet
        .db
        .unsynced_external_p2_puzzle_hashes()
        .await?
        .into_iter()
        .collect();

    let (unsynced, p2_puzzle_hashes): (Vec<Bytes32>, Vec<Bytes32>) = wallet
        .db
        .custody_p2_puzzle_hashes()
        .await?
        .into_iter()
        .partition(|p2_puzzle_hash| unsynced.contains(p2_puzzle_hash));

    let (start_height, start_header_hash) = if delta_sync {
        wallet.db.latest_peak().await?
    } else {
        None
    }
    .map_or_else(
        || (None, wallet.genesis_challenge),
        |(peak, header_hash)| (Some(peak), header_hash),
    );

    let coin_ids = wallet.db.subscription_coin_ids().await?;

    sync_coin_ids(
        &wallet,
        &peer,
        start_height,
        start_header_hash,
        coin_ids,
        sync_sender.clone(),
        command_sender.clone(),
        false,
    )
    .await?;

    for batch in p2_puzzle_hashes.chunks(1000) {
        sync_puzzle_hashes(
            &wallet,
            &peer,
            start_height,
            start_header_hash,
            batch,
            sync_sender.clone(),
            command_sender.clone(),
        )
        .await?;
    }

    for batch in unsynced.chunks(1000) {
        sync_puzzle_hashes(
            &wallet,
            &peer,
            None,
            wallet.genesis_challenge,
            batch,
            sync_sender.clone(),
            command_sender.clone(),
        )
        .await?;
    }

    loop {
        let mut tx = wallet.db.tx().await?;
        let derivations = auto_insert_unhardened_derivations(&wallet, &mut tx).await?;
        let next_index = tx.derivation_index(false).await?;
        tx.commit().await?;

        if derivations.is_empty() {
            break;
        }

        info!("Inserted {} derivations", derivations.len());

        sync_sender
            .send(SyncEvent::DerivationIndex { next_index })
            .await
            .ok();

        for batch in derivations.chunks(1000) {
            sync_puzzle_hashes(
                &wallet,
                &peer,
                None,
                wallet.genesis_challenge,
                batch,
                sync_sender.clone(),
                command_sender.clone(),
            )
            .await?;
        }
    }

    if delta_sync {
        if let Some((height, header_hash)) = state.lock().await.peak_of(peer.socket_addr().ip()) {
            info!(
                "Updating peak from peer to {} with header hash {}",
                height, header_hash
            );

            wallet
                .db
                .insert_block(height, header_hash, None, true)
                .await?;
        } else {
            warn!("No peak found");
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn sync_coin_ids(
    wallet: &Wallet,
    peer: &WalletPeer,
    start_height: Option<u32>,
    start_header_hash: Bytes32,
    coin_ids: Vec<Bytes32>,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
    only_send_event_if_spent: bool,
) -> Result<(), WalletError> {
    for (i, coin_ids) in coin_ids.chunks(10000).enumerate() {
        if i != 0 {
            sleep(Duration::from_millis(500)).await;
        }

        info!(
            "Subscribing to {} coins from peer {}",
            coin_ids.len(),
            peer.socket_addr()
        );

        let coin_states = peer
            .subscribe_coins(coin_ids.to_vec(), start_height, start_header_hash)
            .await?;

        info!("Received {} coin states", coin_states.len());

        if coin_states
            .iter()
            .any(|cs| cs.spent_height.is_some() || !only_send_event_if_spent)
        {
            incremental_sync(wallet, coin_states, true, &sync_sender, &command_sender).await?;
        }
    }

    Ok(())
}

async fn sync_puzzle_hashes(
    wallet: &Wallet,
    peer: &WalletPeer,
    start_height: Option<u32>,
    start_header_hash: Bytes32,
    puzzle_hashes: &[Bytes32],
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
) -> Result<(), WalletError> {
    if puzzle_hashes.is_empty() {
        return Ok(());
    }

    let mut prev_height = start_height;
    let mut prev_header_hash = start_header_hash;

    loop {
        info!(
            "Subscribing to {} puzzle hashes at height {:?} and header hash {} from peer {}",
            puzzle_hashes.len(),
            prev_height,
            prev_header_hash,
            peer.socket_addr()
        );

        let data = peer
            .subscribe_puzzles(
                puzzle_hashes.to_vec(),
                prev_height,
                prev_header_hash,
                CoinStateFilters::new(true, true, true, 0),
            )
            .await?;

        info!("Received {} coin states", data.coin_states.len());

        if !data.coin_states.is_empty() {
            incremental_sync(
                wallet,
                data.coin_states,
                true,
                &sync_sender,
                &command_sender,
            )
            .await?;
        }

        prev_height = Some(data.height);
        prev_header_hash = data.header_hash;

        if data.is_finished {
            break;
        }
    }

    Ok(())
}

pub async fn incremental_sync(
    wallet: &Wallet,
    coin_states: Vec<CoinState>,
    derive_automatically: bool,
    sync_sender: &mpsc::Sender<SyncEvent>,
    command_sender: &mpsc::Sender<SyncCommand>,
) -> Result<(), WalletError> {
    let mut tx = wallet.db.tx().await?;
    let mut confirmed_transactions = HashSet::new();

    for &coin_state in &coin_states {
        if let Some(height) = coin_state.created_height {
            tx.insert_height(height).await?;
        }

        if let Some(height) = coin_state.spent_height {
            tx.insert_height(height).await?;
        }

        tx.insert_coin(coin_state).await?;

        if tx
            .is_custody_p2_puzzle_hash(coin_state.coin.puzzle_hash)
            .await?
        {
            tx.update_coin(
                coin_state.coin.coin_id(),
                Bytes32::default(),
                coin_state.coin.puzzle_hash,
            )
            .await?;
        }

        if let Some(height) = coin_state.created_height {
            // A coin can only be created by spending its parent, so mark the parent spent now
            // rather than letting it look unspent until its own coin state arrives. Other inputs
            // of our mempool items can't be assumed spent, since a different spend bundle may
            // have created this coin.
            tx.mark_coin_spent(coin_state.coin.parent_coin_info, height)
                .await?;

            confirmed_transactions.extend(
                tx.mempool_items_for_output(coin_state.coin.coin_id())
                    .await?,
            );
        }

        if coin_state.spent_height.is_some() {
            confirmed_transactions.extend(
                tx.mempool_items_for_input(coin_state.coin.coin_id())
                    .await?,
            );
        }
    }

    for mempool_item_id in confirmed_transactions {
        tx.remove_mempool_item(mempool_item_id).await?;
    }

    let mut new_derivations = Vec::new();

    if derive_automatically {
        new_derivations = auto_insert_unhardened_derivations(wallet, &mut tx).await?;
    }

    let next_index = tx.derivation_index(false).await?;

    tx.commit().await?;

    if !coin_states.is_empty() {
        sync_sender.send(SyncEvent::CoinsUpdated).await.ok();
    }

    if !new_derivations.is_empty() {
        sync_sender
            .send(SyncEvent::DerivationIndex { next_index })
            .await
            .ok();

        // The sync manager calls this inline while handling a CoinStateUpdate, and it is the
        // only reader of `command_sender`. Awaiting a send on a full channel here would wait for
        // a reader that is blocked on this very send, freezing sync (and every RPC that sends a
        // command while holding the Sage lock) until restart. Hand a full channel off instead.
        let command = SyncCommand::SubscribePuzzles {
            puzzle_hashes: new_derivations,
        };
        if let Err(mpsc::error::TrySendError::Full(command)) = command_sender.try_send(command) {
            warn!(
                "Sync command queue is full; deferring a puzzle subscription so the sync manager \
                 is not left waiting on its own queue"
            );
            let command_sender = command_sender.clone();
            tokio::spawn(async move {
                command_sender.send(command).await.ok();
            });
        }
    }

    Ok(())
}

async fn auto_insert_unhardened_derivations(
    wallet: &Wallet,
    tx: &mut DatabaseTx<'_>,
) -> Result<Vec<Bytes32>, WalletError> {
    // Only BLS wallets derive addresses. Watch wallets sync just their external p2 puzzles.
    if !matches!(wallet.kind, WalletKind::Bls { .. }) {
        return Ok(Vec::new());
    }

    let mut derivations = Vec::new();
    let mut next_index = tx.derivation_index(false).await?;

    let max_index = tx.unused_derivation_index(false).await?;

    while max_index + 500 >= next_index {
        derivations.extend(
            wallet
                .insert_unhardened_derivations(tx, next_index..next_index + 500)
                .await?,
        );

        next_index += 500;
    }

    Ok(derivations)
}

pub async fn add_new_subscriptions(
    wallet: &Wallet,
    peer: &WalletPeer,
    coin_ids: Vec<Bytes32>,
    puzzle_hashes: Vec<Bytes32>,
    sync_sender: mpsc::Sender<SyncEvent>,
    command_sender: mpsc::Sender<SyncCommand>,
) -> Result<(), WalletError> {
    for batch in coin_ids.chunks(1000) {
        sync_coin_ids(
            wallet,
            peer,
            None,
            wallet.genesis_challenge,
            batch.to_vec(),
            sync_sender.clone(),
            command_sender.clone(),
            true,
        )
        .await?;
    }

    for batch in puzzle_hashes.chunks(1000) {
        sync_puzzle_hashes(
            wallet,
            peer,
            None,
            wallet.genesis_challenge,
            batch,
            sync_sender.clone(),
            command_sender.clone(),
        )
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::chia::bls::master_to_wallet_unhardened_intermediate;
    use sage_database::{Database, SharedDatabase};
    use sqlx::{SqlitePool, migrate};
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn incremental_sync_does_not_wait_on_a_full_command_queue() -> anyhow::Result<()> {
        let pool = SqlitePool::connect("file:incremental_sync_full_queue?mode=memory&cache=shared")
            .await?;
        migrate!("../../migrations").run(&pool).await?;
        let db = Database::new(pool);
        db.run_rust_migrations("TXCH".to_string()).await?;

        let shared_pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate!("../../migrations").run(&shared_pool).await?;

        let pk = BlsPair::default().sk.public_key();
        let wallet = Wallet::new(
            db,
            SharedDatabase::new(shared_pool),
            pk.get_fingerprint(),
            WalletKind::Bls {
                intermediate_pk: master_to_wallet_unhardened_intermediate(&pk),
            },
            TESTNET11_CONSTANTS.genesis_challenge,
            AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            None,
        );

        // The sync manager is the queue's only reader, so while it runs incremental_sync
        // nothing drains the queue. A full queue must not stop it.
        let (command_sender, mut command_receiver) = mpsc::channel(1);
        command_sender
            .send(SyncCommand::SetDiscoverPeers(true))
            .await?;
        let (event_sender, _event_receiver) = mpsc::channel(10);

        // A wallet with no derivations always derives new ones, so this reaches the send.
        timeout(
            Duration::from_secs(10),
            incremental_sync(&wallet, Vec::new(), true, &event_sender, &command_sender),
        )
        .await
        .expect("incremental_sync waited on the full command queue")?;

        assert!(matches!(
            command_receiver.recv().await,
            Some(SyncCommand::SetDiscoverPeers(true))
        ));
        assert!(matches!(
            timeout(Duration::from_secs(10), command_receiver.recv()).await?,
            Some(SyncCommand::SubscribePuzzles { puzzle_hashes }) if !puzzle_hashes.is_empty()
        ));

        Ok(())
    }

    #[tokio::test]
    async fn watch_wallets_never_derive_or_spend() -> anyhow::Result<()> {
        let pool = SqlitePool::connect("file:watch_never_derives?mode=memory&cache=shared").await?;
        migrate!("../../migrations").run(&pool).await?;
        let db = Database::new(pool);
        db.run_rust_migrations("TXCH".to_string()).await?;

        let shared_pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate!("../../migrations").run(&shared_pool).await?;

        let watched = Bytes32::new([2; 32]);
        let mut tx = db.tx().await?;
        tx.insert_external_p2_puzzle(watched).await?;
        tx.commit().await?;

        let wallet = Wallet::new(
            db,
            SharedDatabase::new(shared_pool),
            1,
            WalletKind::Watch,
            TESTNET11_CONSTANTS.genesis_challenge,
            AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
            None,
        );

        // Watched puzzle hashes count as the wallet's own, so they're synced.
        assert_eq!(wallet.db.custody_p2_puzzle_hashes().await?, [watched]);
        assert!(wallet.db.is_custody_p2_puzzle_hash(watched).await?);

        let (command_sender, mut command_receiver) = mpsc::channel(10);
        let (event_sender, mut event_receiver) = mpsc::channel(10);

        incremental_sync(&wallet, Vec::new(), true, &event_sender, &command_sender).await?;

        assert!(command_receiver.try_recv().is_err(), "it subscribed");
        assert!(event_receiver.try_recv().is_err(), "it sent an event");
        assert_eq!(wallet.db.max_derivation_index(false).await?, None);

        let mut tx = wallet.db.tx().await?;
        assert!(matches!(
            wallet.insert_unhardened_derivations(&mut tx, 0..10).await,
            Err(WalletError::DerivationsNotSupported)
        ));
        tx.commit().await?;

        assert!(matches!(
            wallet.p2_puzzle_hashes(1, false, true).await,
            Err(WalletError::DerivationsNotSupported)
        ));
        assert!(matches!(
            wallet.change_p2_puzzle_hash().await,
            Err(WalletError::WatchOnly)
        ));

        let mut ctx = SpendContext::new();
        assert!(matches!(
            wallet.spend(&mut ctx, Vec::new(), &[]).await,
            Err(WalletError::WatchOnly)
        ));

        Ok(())
    }
}
