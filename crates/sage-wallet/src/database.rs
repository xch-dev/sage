use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use chia::{
    bls::Signature,
    protocol::{Bytes32, CoinState},
};
use sage_database::{
    CatAsset, CoinKind, Database, DatabaseTx, DidCoinInfo, NftCoinInfo, SingletonAsset,
};

use crate::{fetch_nft_did, ChildKind, Transaction, WalletError, WalletPeer};

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
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();
    let db_id = tx.coin_id(coin_id).await?;

    let Some(p2_puzzle_hash) = info.p2_puzzle_hash() else {
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    let Some(p2_puzzle_id) = tx.p2_puzzle_id(p2_puzzle_hash).await? else {
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    match info {
        ChildKind::Launcher | ChildKind::Unknown => {}
        ChildKind::Cat {
            info,
            lineage_proof,
        } => {
            tx.insert_lineage_proof(db_id, lineage_proof).await?;

            let asset_id = tx.insert_cat(CatAsset::empty(info.asset_id, true)).await?;

            tx.sync_coin(db_id, asset_id, p2_puzzle_id, info.hidden_puzzle_hash)
                .await?;
        }
        ChildKind::Did {
            lineage_proof,
            info,
        } => {
            tx.insert_lineage_proof(db_id, lineage_proof).await?;

            let coin_info = DidCoinInfo {
                metadata: info.metadata,
                recovery_list_hash: info.recovery_list_hash,
                num_verifications_required: info.num_verifications_required,
            };

            let asset_id = tx
                .insert_did(
                    SingletonAsset::empty(info.launcher_id, true, coin_state.created_height),
                    &coin_info,
                )
                .await?;

            if tx.is_latest_singleton_coin(coin_id).await? {
                tx.update_did_coin_info(asset_id, &coin_info).await?;
            }

            tx.sync_coin(db_id, asset_id, p2_puzzle_id, None).await?;
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
        } => {
            tx.insert_lineage_proof(db_id, lineage_proof).await?;

            let coin_info = NftCoinInfo {
                minter_hash: minter_did,
                owner_hash: info.current_owner,
                metadata: info.metadata,
                metadata_updater_puzzle_hash: info.metadata_updater_puzzle_hash,
                royalty_puzzle_hash: info.royalty_puzzle_hash,
                royalty_basis_points: info.royalty_basis_points,
                data_hash: metadata.as_ref().and_then(|m| m.data_hash),
                metadata_hash: metadata.as_ref().and_then(|m| m.metadata_hash),
                license_hash: metadata.as_ref().and_then(|m| m.license_hash),
                edition_number: metadata.as_ref().map(|m| m.edition_number),
                edition_total: metadata.as_ref().map(|m| m.edition_total),
            };

            let asset_id = tx
                .insert_nft(
                    SingletonAsset::empty(info.launcher_id, true, coin_state.created_height),
                    &coin_info,
                )
                .await?;

            if tx.is_latest_singleton_coin(coin_id).await? {
                tx.update_nft_coin_info(asset_id, &coin_info).await?;
            }

            tx.sync_coin(db_id, asset_id, p2_puzzle_id, None).await?;

            let (data_uris, metadata_uris, license_uris) = metadata
                .map(|metadata| {
                    (
                        metadata.data_uris,
                        metadata.metadata_uris,
                        metadata.license_uris,
                    )
                })
                .unwrap_or_default();

            if let Some(hash) = coin_info.data_hash {
                let id = tx.insert_file(hash).await?;

                for uri in data_uris {
                    tx.insert_file_uri(id, uri).await?;
                }
            }

            if let Some(hash) = coin_info.metadata_hash {
                let id = tx.insert_file(hash).await?;

                for uri in metadata_uris {
                    tx.insert_file_uri(id, uri).await?;
                }
            }

            if let Some(hash) = coin_info.license_hash {
                let id = tx.insert_file(hash).await?;

                for uri in license_uris {
                    tx.insert_file_uri(id, uri).await?;
                }
            }
        }
    }

    Ok(true)
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
                    ChildKind::Unknown | ChildKind::Launcher => CoinKind::Unknown,
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
