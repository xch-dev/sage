use std::{collections::HashSet, sync::Arc, time::Duration};

use chia::protocol::{Bytes32, CoinState, CoinStateFilters};
use sage_database::DatabaseTx;
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use tracing::{info, warn};

use crate::{SyncCommand, Wallet, WalletError, WalletPeer};

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

    let p2_puzzle_hashes = wallet.db.custody_p2_puzzle_hashes().await?;

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

    for batch in p2_puzzle_hashes.chunks(500) {
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

        for batch in derivations.chunks(500) {
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

        confirmed_transactions.extend(
            tx.mempool_items_for_output(coin_state.coin.coin_id())
                .await?,
        );

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

        command_sender
            .send(SyncCommand::SubscribePuzzles {
                puzzle_hashes: new_derivations,
            })
            .await
            .ok();
    }

    Ok(())
}

async fn auto_insert_unhardened_derivations(
    wallet: &Wallet,
    tx: &mut DatabaseTx<'_>,
) -> Result<Vec<Bytes32>, WalletError> {
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
    sync_coin_ids(
        wallet,
        peer,
        None,
        wallet.genesis_challenge,
        coin_ids,
        sync_sender.clone(),
        command_sender.clone(),
        true,
    )
    .await?;

    sync_puzzle_hashes(
        wallet,
        peer,
        None,
        wallet.genesis_challenge,
        &puzzle_hashes,
        sync_sender.clone(),
        command_sender.clone(),
    )
    .await?;

    Ok(())
}
