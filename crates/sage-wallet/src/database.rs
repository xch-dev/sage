use std::collections::{HashMap, HashSet};

use chia::{
    bls::Signature,
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, CoinState, Program},
    puzzles::{nft::NftMetadata, LineageProof},
};
use chia_wallet_sdk::driver::NftInfo;
use sage_assets::base64_data_uri;
use sage_database::{Asset, AssetKind, Database, DatabaseTx, DidCoinInfo, NftCoinInfo};
use tracing::warn;

use crate::{compute_nft_info, fetch_nft_did, ChildKind, Transaction, WalletError, WalletPeer};

pub async fn insert_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    info: ChildKind,
    minter_hash: Option<Bytes32>,
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();

    let custody_p2_puzzle_hashes = info.custody_p2_puzzle_hashes();

    let mut is_relevant = false;

    for custody_p2_puzzle_hash in custody_p2_puzzle_hashes {
        if tx.is_custody_p2_puzzle_hash(custody_p2_puzzle_hash).await? {
            is_relevant = true;
            break;
        }
    }

    if !is_relevant {
        warn!(
            "Deleting coin {} because it isn't relevant to this wallet",
            coin_id
        );
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    }

    match info {
        ChildKind::Launcher | ChildKind::Unknown => {
            warn!("Deleting coin {} because it has an unknown puzzle", coin_id);
            tx.delete_coin(coin_id).await?;
            return Ok(false);
        }
        ChildKind::Clawback { info } => {
            tx.insert_clawback_p2_puzzle(info).await?;

            tx.update_coin(coin_id, Bytes32::default(), info.tree_hash().into(), None)
                .await?;
        }
        ChildKind::Cat {
            info,
            lineage_proof,
            clawback,
        } => {
            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            tx.insert_asset(Asset {
                hash: info.asset_id,
                name: None,
                ticker: None,
                precision: 3,
                icon_url: None,
                description: None,
                is_sensitive_content: false,
                is_visible: true,
                kind: AssetKind::Token,
            })
            .await?;

            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            tx.update_coin(
                coin_id,
                info.asset_id,
                info.p2_puzzle_hash,
                info.hidden_puzzle_hash,
            )
            .await?;
        }
        ChildKind::Did {
            lineage_proof,
            info,
            clawback,
        } => {
            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            let coin_info = DidCoinInfo {
                metadata: info.metadata,
                recovery_list_hash: info.recovery_list_hash,
                num_verifications_required: info.num_verifications_required,
            };

            tx.insert_asset(Asset {
                hash: info.launcher_id,
                name: None,
                ticker: None,
                precision: 1,
                icon_url: None,
                description: None,
                is_sensitive_content: false,
                is_visible: true,
                kind: AssetKind::Did,
            })
            .await?;

            tx.insert_did(info.launcher_id, &coin_info).await?;

            if coin_state.spent_height.is_none() {
                tx.update_did(info.launcher_id, &coin_info).await?;
            }

            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            tx.update_coin(coin_id, info.launcher_id, info.p2_puzzle_hash, None)
                .await?;
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
            clawback,
        } => {
            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            insert_nft(
                tx,
                coin_state,
                Some(lineage_proof),
                info,
                metadata,
                minter_hash,
            )
            .await?;
        }
    }

    Ok(true)
}

pub async fn insert_nft(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    lineage_proof: Option<LineageProof>,
    info: NftInfo<Program>,
    metadata: Option<NftMetadata>,
    minter_hash: Option<Bytes32>,
) -> Result<(), WalletError> {
    if let Some(lineage_proof) = lineage_proof {
        tx.insert_lineage_proof(coin_state.coin.coin_id(), lineage_proof)
            .await?;
    }

    let icon_url =
        if let Some(data_hash) = metadata.as_ref().and_then(|metadata| metadata.data_hash) {
            tx.icon(data_hash).await?.map(|icon| {
                base64_data_uri(
                    &icon.data,
                    &icon.mime_type.unwrap_or_else(|| "image/png".to_string()),
                )
            })
        } else {
            None
        };

    let mut asset = Asset {
        hash: info.launcher_id,
        name: None,
        ticker: None,
        precision: 1,
        icon_url,
        description: None,
        is_sensitive_content: false,
        is_visible: true,
        kind: AssetKind::Nft,
    };

    let mut coin_info = NftCoinInfo {
        collection_hash: Bytes32::default(),
        collection_name: None,
        minter_hash,
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
            let computed = compute_nft_info(minter_hash, &blob);
            asset.name = computed.name;
            asset.description = computed.description;
            asset.is_sensitive_content = computed.sensitive_content;

            if let Some(collection) = computed.collection {
                coin_info.collection_hash = collection.hash;
                tx.insert_collection(collection).await?;
            }
        }
    };

    tx.insert_asset(asset).await?;

    tx.insert_nft(info.launcher_id, &coin_info).await?;

    if coin_state.spent_height.is_none() || lineage_proof.is_none() {
        tx.update_nft(info.launcher_id, &coin_info).await?;
    }

    if lineage_proof.is_some() {
        tx.update_coin(
            coin_state.coin.coin_id(),
            info.launcher_id,
            info.p2_puzzle_hash,
            None,
        )
        .await?;
    }

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

    for (index, input) in transaction.inputs.iter().enumerate() {
        let input_coin_id = input.coin_spend.coin.coin_id();

        // Insert the spend into the database in the proper order so it can be reconstructed later.
        tx.insert_mempool_spend(transaction_id, input.coin_spend.clone(), index)
            .await?;

        // If the coin isn't ephemeral (exists on-chain) and we already have it in the database,
        // we can attach it to the transaction as our coin for display purposes.
        if !output_coin_ids.contains(&input_coin_id) && tx.is_known_coin(input_coin_id).await? {
            tx.insert_mempool_coin(transaction_id, input_coin_id, true, false)
                .await?;
        }

        for output in &input.outputs {
            // Coins that don't exist on-chain yet don't have a created or spent height.
            let coin_state = CoinState::new(output.coin, None, None);
            let coin_id = output.coin.coin_id();

            // If it's an XCH coin, we can insert it and sync it immediately.
            // Attach it to the transaction as an output for display purposes.
            if tx
                .is_custody_p2_puzzle_hash(output.coin.puzzle_hash)
                .await?
            {
                tx.insert_coin(coin_state).await?;

                tx.update_coin(coin_id, Bytes32::default(), output.coin.puzzle_hash, None)
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
            let custody_p2_puzzle_hashes = output.kind.custody_p2_puzzle_hashes();

            let mut is_relevant = false;

            for custody_p2_puzzle_hash in custody_p2_puzzle_hashes {
                if tx.is_custody_p2_puzzle_hash(custody_p2_puzzle_hash).await? {
                    is_relevant = true;
                    break;
                }
            }

            if !is_relevant {
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
                output.kind.clone(),
                minter_dids.get(&output.coin.coin_id()).copied(),
            )
            .await?;
        }
    }

    for input in transaction.inputs {
        // We are inserting the children as part of inserting the transaction, so we don't need to do it again
        tx.set_children_synced(input.coin_spend.coin.coin_id())
            .await?;
    }

    tx.commit().await?;

    Ok(subscriptions)
}
