use std::collections::{HashMap, HashSet};

use chia::{
    bls::Signature,
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, CoinState},
};
use sage_database::{Asset, CatAsset, Database, DatabaseTx, DidCoinInfo, NftCoinInfo};
use tracing::warn;

use crate::{compute_nft_info, fetch_nft_did, ChildKind, Transaction, WalletError, WalletPeer};

pub async fn insert_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    info: ChildKind,
    minter_did: Option<Bytes32>,
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();

    let Some(custody_p2_puzzle_hash) = info.custody_p2_puzzle_hash() else {
        warn!(
            "Deleting coin {} because it has no custody p2 puzzle hash",
            coin_id
        );
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    if !tx.is_p2_puzzle_hash(custody_p2_puzzle_hash).await? {
        warn!(
            "Deleting coin {} because it has a custody p2 puzzle hash we don't know about",
            coin_id
        );
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    match info {
        ChildKind::Launcher | ChildKind::Unknown => {
            warn!("Deleting coin {} because it has an unknown puzzle", coin_id);
            tx.delete_coin(coin_id).await?;
            return Ok(false);
        }
        ChildKind::Clawback { info } => {
            tx.insert_clawback_p2_puzzle(info).await?;

            tx.sync_coin(coin_id, Bytes32::default(), info.tree_hash().into(), None)
                .await?;
        }
        ChildKind::Cat {
            info,
            lineage_proof,
        } => {
            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            tx.insert_cat(CatAsset {
                asset: Asset::empty(info.asset_id, true, None),
                ticker: None,
            })
            .await?;

            tx.sync_coin(
                coin_id,
                info.asset_id,
                custody_p2_puzzle_hash,
                info.hidden_puzzle_hash,
            )
            .await?;
        }
        ChildKind::Did {
            lineage_proof,
            info,
        } => {
            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            let coin_info = DidCoinInfo {
                metadata: info.metadata,
                recovery_list_hash: info.recovery_list_hash,
                num_verifications_required: info.num_verifications_required,
            };

            tx.insert_did(
                Asset::empty(info.launcher_id, true, coin_state.created_height),
                &coin_info,
            )
            .await?;

            if tx.is_latest_singleton_coin(coin_id).await? {
                tx.update_did_coin_info(info.launcher_id, &coin_info)
                    .await?;
            }

            tx.sync_coin(coin_id, info.launcher_id, custody_p2_puzzle_hash, None)
                .await?;
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
        } => {
            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            let mut asset = Asset::empty(info.launcher_id, true, coin_state.created_height);
            let mut coin_info = NftCoinInfo {
                collection_hash: Bytes32::default(),
                collection_name: None,
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

            if let Some(metadata_hash) = &metadata.as_ref().and_then(|m| m.metadata_hash) {
                if let Some(blob) = tx.file_data(*metadata_hash).await? {
                    let computed = compute_nft_info(minter_did, &blob);
                    asset.name = computed.name;
                    asset.description = computed.description;
                    asset.is_sensitive_content = computed.sensitive_content;

                    if let Some(collection) = computed.collection {
                        coin_info.collection_hash = collection.hash;
                        tx.insert_collection(collection).await?;
                    }
                }
            };

            tx.insert_nft(asset, &coin_info).await?;

            if tx.is_latest_singleton_coin(coin_id).await? {
                tx.update_nft_coin_info(info.launcher_id, &coin_info)
                    .await?;
            }

            tx.sync_coin(coin_id, info.launcher_id, custody_p2_puzzle_hash, None)
                .await?;

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
                tx.insert_file(hash).await?;

                for uri in data_uris {
                    tx.insert_file_uri(hash, uri).await?;
                }
            }

            if let Some(hash) = coin_info.metadata_hash {
                tx.insert_file(hash).await?;

                for uri in metadata_uris {
                    tx.insert_file_uri(hash, uri).await?;
                }
            }

            if let Some(hash) = coin_info.license_hash {
                tx.insert_file(hash).await?;

                for uri in license_uris {
                    tx.insert_file_uri(hash, uri).await?;
                }
            }
        }
    }

    Ok(true)
}

pub async fn insert_transaction(
    db: &Database,
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    transaction_id: Bytes32,
    transaction: Transaction,
    aggregated_signature: Signature,
) -> Result<Vec<Bytes32>, WalletError> {
    // Make lookups faster for inputs and outputs, and prepare pending coin spends.
    let mut coin_spends = HashMap::new();
    let mut output_coin_ids = HashSet::new();

    for input in &transaction.inputs {
        coin_spends.insert(input.coin_spend.coin.coin_id(), input.coin_spend.clone());

        for output in &input.outputs {
            output_coin_ids.insert(output.coin.coin_id());
        }
    }

    // Fetch minter DIDs for created NFTs in the transaction from the blockchain.
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

    // Insert the transaction into the database.
    let mut tx = db.tx().await?;

    tx.insert_mempool_item(transaction_id, aggregated_signature, transaction.fee)
        .await?;

    let mut subscriptions = Vec::new();

    for (index, input) in transaction.inputs.into_iter().enumerate() {
        let input_coin_id = input.coin_spend.coin.coin_id();

        // Insert the spend into the database in the proper order so it can be reconstructed later.
        tx.insert_mempool_spend(transaction_id, input.coin_spend, index)
            .await?;

        // If the coin isn't ephemeral (exists on-chain) and we already have it in the database,
        // we can attach it to the transaction as our coin for display purposes.
        if !output_coin_ids.contains(&input_coin_id) && tx.is_known_coin(input_coin_id).await? {
            tx.insert_mempool_coin(transaction_id, input_coin_id, true, false)
                .await?;
        }

        for output in input.outputs {
            // Coins that don't exist on-chain yet don't have a created or spent height.
            let coin_state = CoinState::new(output.coin, None, None);
            let coin_id = output.coin.coin_id();

            // If it's an XCH coin, we can insert it and sync it immediately.
            // Attach it to the transaction as an output for display purposes.
            if tx.is_p2_puzzle_hash(output.coin.puzzle_hash).await? {
                tx.insert_coin(coin_state).await?;

                tx.sync_coin(coin_id, Bytes32::default(), output.coin.puzzle_hash, None)
                    .await?;

                tx.insert_mempool_coin(
                    transaction_id,
                    coin_id,
                    coin_spends.contains_key(&coin_id),
                    true,
                )
                .await?;

                continue;
            }

            // We don't want to insert output coins that we won't own in the future.
            let Some(custody_p2_puzzle_hash) = output.kind.custody_p2_puzzle_hash() else {
                continue;
            };

            if !tx.is_p2_puzzle_hash(custody_p2_puzzle_hash).await? {
                continue;
            }

            // Insert the coin into the database and attach it to the transaction as an output for display purposes.
            tx.insert_coin(coin_state).await?;

            tx.insert_mempool_coin(
                transaction_id,
                coin_id,
                coin_spends.contains_key(&coin_id),
                true,
            )
            .await?;

            // We should subscribe to the coin so we know when it's created on-chain.
            // TODO: Is this necessary? Subscribing to the p2 puzzle hash is probably sufficient for created coins.
            if output.kind.subscribe() {
                subscriptions.push(coin_id);
            }

            // Do the busy work of inserting the asset information into the database now that the coin exists.
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
