use std::collections::HashMap;

use chia::{
    bls::Signature,
    protocol::{Bytes32, CoinState},
};
use sage_database::{Asset, CatAsset, CoinKind, Database, DatabaseTx, DidCoinInfo, NftCoinInfo};

use crate::{compute_nft_info, fetch_nft_did, ChildKind, Transaction, WalletError, WalletPeer};

pub async fn insert_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    info: ChildKind,
    minter_did: Option<Bytes32>,
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();

    let Some(p2_puzzle_hash) = info.p2_puzzle_hash() else {
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    if !tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    };

    match info {
        ChildKind::Launcher | ChildKind::Unknown => {}
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
                p2_puzzle_hash,
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

            tx.sync_coin(coin_id, info.launcher_id, p2_puzzle_hash, None)
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
                collection_id: None,
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
                        coin_info.collection_id = Some(collection.hash);
                        tx.insert_collection(collection).await?;
                    }
                }
            };

            tx.insert_nft(asset, &coin_info).await?;

            if tx.is_latest_singleton_coin(coin_id).await? {
                tx.update_nft_coin_info(info.launcher_id, &coin_info)
                    .await?;
            }

            tx.sync_coin(coin_id, info.launcher_id, p2_puzzle_hash, None)
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
