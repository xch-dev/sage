use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use chia::{
    bls::Signature,
    protocol::{Bytes32, CoinState},
};
use sage_database::{CatRow, CoinKind, Database, DatabaseTx, DidRow, NftRow};

use crate::{compute_nft_info, fetch_nft_did, ChildKind, Transaction, WalletError, WalletPeer};

#[derive(Debug, Default, Clone, Copy)]
pub struct UpsertCounters {
    pub is_p2: Duration,
    pub insert_coin_state: Duration,
    pub update_coin_state: Duration,
    pub insert_p2_coin: Duration,
    pub update_created_puzzle: Duration,
    pub delete_puzzle: Duration,
}

pub async fn upsert_coin(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    transaction_id: Option<Bytes32>,
    counters: &mut UpsertCounters,
) -> Result<(), WalletError> {
    let coin_id = coin_state.coin.coin_id();

    // Check if the coin is plain XCH, rather than an asset that wraps the p2 puzzle hash.
    let start = Instant::now();
    let is_p2 = tx.is_p2_puzzle_hash(coin_state.coin.puzzle_hash).await?;
    counters.is_p2 += start.elapsed();

    // If the coin is XCH, there's no reason to sync the puzzle.
    let start = Instant::now();
    tx.insert_coin_state(coin_state, is_p2, transaction_id)
        .await?;
    counters.insert_coin_state += start.elapsed();

    // If the coin already existed, instead of replacing it we will just update it.
    let start = Instant::now();
    tx.update_coin_state(
        coin_id,
        coin_state.created_height,
        coin_state.spent_height,
        transaction_id,
    )
    .await?;
    counters.update_coin_state += start.elapsed();

    // This allows querying for XCH coins without joining on the derivations table.
    if is_p2 {
        let start = Instant::now();
        tx.insert_p2_coin(coin_id).await?;
        counters.insert_p2_coin += start.elapsed();
    } else {
        let start = Instant::now();
        update_created_puzzle(tx, coin_state).await?;
        counters.update_created_puzzle += start.elapsed();
    }

    Ok(())
}

pub async fn insert_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    info: ChildKind,
    minter_did: Option<Bytes32>,
) -> Result<(), WalletError> {
    let coin_id = coin_state.coin.coin_id();

    match info {
        ChildKind::Launcher | ChildKind::Unknown { .. } => {}
        ChildKind::Cat {
            asset_id,
            lineage_proof,
            p2_puzzle_hash,
        } => {
            tx.sync_coin(coin_id, Some(p2_puzzle_hash), CoinKind::Cat)
                .await?;
            tx.insert_cat(CatRow {
                asset_id,
                name: None,
                ticker: None,
                description: None,
                icon: None,
                visible: true,
                fetched: false,
            })
            .await?;
            tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                .await?;
        }
        ChildKind::Did {
            lineage_proof,
            info,
        } => {
            let launcher_id = info.launcher_id;

            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash), CoinKind::Did)
                .await?;
            tx.insert_did_coin(coin_id, lineage_proof, info).await?;

            if coin_state.spent_height.is_some() {
                return Ok(());
            }

            let name = tx.get_future_did_name(launcher_id).await?;

            if name.is_some() {
                tx.delete_future_did_name(launcher_id).await?;
            }

            let mut row = tx.did_row(launcher_id).await?.unwrap_or(DidRow {
                launcher_id,
                coin_id,
                name,
                is_owned: coin_state.spent_height.is_none(),
                visible: true,
                created_height: coin_state.created_height,
            });

            if coin_state.spent_height.is_none() {
                row.is_owned = true;
            }

            row.coin_id = coin_id;
            row.created_height = coin_state.created_height;

            tx.insert_did(row).await?;
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
        } => {
            let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
            let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
            let license_hash = metadata.as_ref().and_then(|m| m.license_hash);
            let launcher_id = info.launcher_id;
            let owner_did = info.current_owner;

            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash), CoinKind::Nft)
                .await?;

            tx.insert_nft_coin(
                coin_id,
                lineage_proof,
                info,
                data_hash,
                metadata_hash,
                license_hash,
            )
            .await?;

            if coin_state.spent_height.is_some() {
                return Ok(());
            }

            let mut row = tx.nft_row(launcher_id).await?.unwrap_or(NftRow {
                launcher_id,
                coin_id,
                collection_id: None,
                minter_did,
                owner_did,
                visible: true,
                sensitive_content: false,
                name: None,
                is_owned: coin_state.spent_height.is_none(),
                created_height: coin_state.created_height,
                metadata_hash,
            });

            if coin_state.spent_height.is_none() {
                row.is_owned = true;
            }

            let metadata_blob = if let Some(metadata_hash) = metadata_hash {
                tx.fetch_nft_data(metadata_hash)
                    .await?
                    .map(|data| data.blob)
            } else {
                None
            };

            let computed_info = compute_nft_info(minter_did, metadata_blob.as_deref());

            row.coin_id = coin_id;
            row.sensitive_content = computed_info.sensitive_content;
            row.name = computed_info.name;
            row.collection_id = computed_info
                .collection
                .as_ref()
                .map(|col| col.collection_id);

            if let Some(collection) = computed_info.collection {
                tx.insert_collection(collection).await?;
            }

            row.owner_did = owner_did;
            row.created_height = coin_state.created_height;

            tx.insert_nft(row).await?;

            if let Some(metadata) = metadata {
                if let Some(hash) = data_hash {
                    for uri in metadata.data_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }

                if let Some(hash) = metadata_hash {
                    for uri in metadata.metadata_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }

                if let Some(hash) = license_hash {
                    for uri in metadata.license_uris {
                        tx.insert_nft_uri(uri, hash).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_puzzle(tx: &mut DatabaseTx<'_>, coin_id: Bytes32) -> Result<(), WalletError> {
    tx.set_did_not_owned(coin_id).await?;
    tx.set_nft_not_owned(coin_id).await?;
    Ok(())
}

pub async fn update_created_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
) -> Result<(), WalletError> {
    let coin_id = coin_state.coin.coin_id();

    tx.set_did_created_height(coin_id, coin_state.created_height)
        .await?;

    tx.set_nft_created_height(coin_id, coin_state.created_height)
        .await?;

    Ok(())
}

pub async fn insert_transaction(
    db: &Database,
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    transaction_id: Bytes32,
    transaction: Transaction,
    aggregated_signature: Signature,
) -> Result<Vec<Bytes32>, WalletError> {
    let mut coin_spends = HashMap::new();

    for input in &transaction.inputs {
        coin_spends.insert(input.coin_spend.coin.coin_id(), input.coin_spend.clone());
    }

    let mut minter_dids = HashMap::new();

    for input in &transaction.inputs {
        for output in &input.outputs {
            let ChildKind::Nft { info, .. } = &output.kind else {
                continue;
            };

            if let Some(did_id) =
                fetch_nft_did(peer, genesis_challenge, info.launcher_id, &coin_spends).await?
            {
                minter_dids.insert(output.coin.coin_id(), did_id);
            }
        }
    }

    let mut tx = db.tx().await?;

    tx.insert_pending_transaction(transaction_id, aggregated_signature, transaction.fee)
        .await?;

    for coin_id in transaction
        .inputs
        .iter()
        .map(|input| input.coin_spend.coin.coin_id())
    {
        delete_puzzle(&mut tx, coin_id).await?;
    }

    let mut subscriptions = Vec::new();

    for (index, input) in transaction.inputs.into_iter().enumerate() {
        tx.insert_transaction_spend(transaction_id, input.coin_spend, index)
            .await?;

        for output in input.outputs {
            let coin_state = CoinState::new(output.coin, None, None);
            let coin_id = output.coin.coin_id();

            if tx.is_p2_puzzle_hash(output.coin.puzzle_hash).await? {
                tx.insert_coin_state(coin_state, true, Some(transaction_id))
                    .await?;
                tx.insert_p2_coin(coin_id).await?;
                continue;
            }

            let Some(p2_puzzle_hash) = output.kind.p2_puzzle_hash() else {
                continue;
            };

            if !tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                continue;
            }

            tx.insert_coin_state(coin_state, true, Some(transaction_id))
                .await?;
            tx.sync_coin(
                coin_id,
                Some(p2_puzzle_hash),
                match output.kind {
                    ChildKind::Unknown { .. } | ChildKind::Launcher => CoinKind::Unknown,
                    ChildKind::Cat { .. } => CoinKind::Cat,
                    ChildKind::Did { .. } => CoinKind::Did,
                    ChildKind::Nft { .. } => CoinKind::Nft,
                },
            )
            .await?;

            if output.kind.subscribe() {
                subscriptions.push(coin_id);
            }

            insert_puzzle(
                &mut tx,
                coin_state,
                output.kind,
                minter_dids.get(&output.coin.coin_id()).copied(),
            )
            .await?;
        }
    }

    tx.commit().await?;

    Ok(subscriptions)
}

pub async fn safely_remove_transaction(
    tx: &mut DatabaseTx<'_>,
    transaction_id: Bytes32,
) -> Result<(), WalletError> {
    for coin_id in tx.transaction_coin_ids(transaction_id).await? {
        if tx.is_p2_coin(coin_id).await? == Some(false) {
            tx.unsync_coin(coin_id).await?;
        }
    }

    tx.remove_transaction(transaction_id).await?;

    Ok(())
}
