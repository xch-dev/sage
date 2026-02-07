use std::collections::{HashMap, HashSet};

use chia_wallet_sdk::{
    chia::puzzle_types::{nft::NftMetadata, LineageProof},
    prelude::*,
};
use sage_assets::base64_data_uri;
use sage_database::{
    Asset, AssetKind, Database, DatabaseTx, DidCoinInfo, NftCoinInfo, OptionCoinInfo,
    SerializedNftInfo,
};
use tracing::{error, warn};

use crate::{
    compute_nft_info, ChildKind, OptionContext, PuzzleContext, Transaction, WalletError, WalletPeer,
};

pub async fn validate_wallet_coin(
    tx: &mut DatabaseTx<'_>,
    coin_id: Bytes32,
    info: &ChildKind,
) -> Result<bool, WalletError> {
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

    Ok(true)
}

pub async fn insert_puzzle(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    info: ChildKind,
    context: PuzzleContext,
    underlying_p2_puzzle_hash: Option<Bytes32>,
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();

    // It's an XCH coin, so we don't care about the child kind
    if let Some(underlying_p2_puzzle_hash) = underlying_p2_puzzle_hash {
        if coin_state.coin.puzzle_hash == underlying_p2_puzzle_hash {
            tx.update_coin(coin_id, Bytes32::default(), underlying_p2_puzzle_hash)
                .await?;
            return Ok(true);
        }
    }

    match info {
        ChildKind::Launcher | ChildKind::Unknown => {
            warn!("Deleting coin {coin_id} because it has an unknown puzzle");
            tx.delete_coin(coin_id).await?;
            return Ok(false);
        }
        ChildKind::Clawback { info } => {
            if underlying_p2_puzzle_hash.is_some() {
                warn!("Deleting underlying coin {coin_id} because clawbacks are unsupported");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            }

            tx.insert_clawback_p2_puzzle(info).await?;

            tx.update_coin(coin_id, Bytes32::default(), info.tree_hash().into())
                .await?;
        }
        ChildKind::Cat {
            info,
            lineage_proof,
            clawback,
        } => {
            if underlying_p2_puzzle_hash.is_some() && clawback.is_some() {
                warn!("Deleting underlying coin {coin_id} because clawbacks are unsupported");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            }

            tx.insert_lineage_proof(coin_id, lineage_proof).await?;

            if let Some(hidden_puzzle_hash) = tx.existing_hidden_puzzle_hash(info.asset_id).await? {
                match (hidden_puzzle_hash, info.hidden_puzzle_hash) {
                    (None, Some(hidden_puzzle_hash)) => {
                        warn!(
                            "Received a CAT coin with hidden puzzle hash {hidden_puzzle_hash}, \
                            but we already have an asset with no hidden puzzle hash. This is a \
                            security risk, so the coin has been deleted."
                        );
                        tx.delete_coin(coin_id).await?;
                        return Ok(false);
                    }
                    (Some(hidden_puzzle_hash), None) => {
                        error!(
                            "Received a CAT coin without a hidden puzzle hash, but we already \
                            synced one or more coins with hidden puzzle hash {hidden_puzzle_hash}. \
                            The existing coins have been immediately deleted as they will pollute \
                            legitimate CAT coins with the same asset id."
                        );
                        tx.delete_asset_coins(info.asset_id).await?;
                    }
                    (None, None) => {}
                    (Some(old), Some(new)) => {
                        if old != new {
                            warn!(
                                "Received a CAT coin with a different hidden puzzle hash than \
                                the one we already have. The new coin has been deleted as it \
                                will pollute legitimate CAT coins with the same asset id."
                            );
                            tx.delete_coin(coin_id).await?;
                            return Ok(false);
                        }
                    }
                }
            }

            tx.insert_asset(Asset {
                hash: info.asset_id,
                name: None,
                ticker: None,
                precision: 3,
                icon_url: None,
                description: None,
                is_sensitive_content: false,
                is_visible: true,
                hidden_puzzle_hash: info.hidden_puzzle_hash,
                kind: AssetKind::Token,
            })
            .await?;

            tx.update_hidden_puzzle_hash(info.asset_id, info.hidden_puzzle_hash)
                .await?;

            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            tx.update_coin(coin_id, info.asset_id, info.p2_puzzle_hash)
                .await?;
        }
        ChildKind::Did {
            lineage_proof,
            info,
            clawback,
        } => {
            if underlying_p2_puzzle_hash.is_some() {
                warn!("Deleting underlying coin {coin_id} because DIDs are unsupported");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            }

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
                hidden_puzzle_hash: None,
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

            tx.update_coin(coin_id, info.launcher_id, info.p2_puzzle_hash)
                .await?;
        }
        ChildKind::Nft {
            lineage_proof,
            info,
            metadata,
            clawback,
        } => {
            if underlying_p2_puzzle_hash.is_some() {
                warn!("Deleting underlying coin {coin_id} because NFTs are unsupported");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            }

            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            insert_nft(tx, coin_state, Some(lineage_proof), info, metadata, context).await?;
        }
        ChildKind::Option {
            lineage_proof,
            info,
            clawback,
        } => {
            if underlying_p2_puzzle_hash.is_some() {
                warn!("Deleting underlying coin {coin_id} because recursive option contracts are unsupported");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            }

            let PuzzleContext::Option(context) = context else {
                warn!("Received an invalid option contract coin (either the metadata or underlying coin were invalid)");
                tx.delete_coin(coin_id).await?;
                return Ok(false);
            };

            if let Some(clawback) = clawback {
                tx.insert_clawback_p2_puzzle(clawback).await?;
            }

            if !insert_option(tx, coin_state, Some(lineage_proof), info, context).await? {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

pub async fn insert_nft(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    lineage_proof: Option<LineageProof>,
    info: SerializedNftInfo,
    metadata: Option<NftMetadata>,
    context: PuzzleContext,
) -> Result<(), WalletError> {
    if let Some(lineage_proof) = lineage_proof {
        tx.insert_lineage_proof(coin_state.coin.coin_id(), lineage_proof)
            .await?;
    }

    let icon_url =
        if let Some(data_hash) = metadata.as_ref().and_then(|metadata| metadata.data_hash) {
            tx.icon(data_hash)
                .await?
                .map(|icon| base64_data_uri(&icon.data, "image/png"))
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
        hidden_puzzle_hash: None,
        kind: AssetKind::Nft,
    };

    let mut coin_info = NftCoinInfo {
        collection_hash: Bytes32::default(),
        collection_name: None,
        minter_hash: match context {
            PuzzleContext::Nft { minter_hash } => minter_hash,
            _ => None,
        },
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
            let computed = compute_nft_info(coin_info.minter_hash, &blob);
            asset.name = computed.name;
            asset.description = computed.description;
            asset.is_sensitive_content = computed.sensitive_content;

            if let Some(collection) = computed.collection {
                coin_info.collection_hash = collection.hash;
                tx.insert_collection(collection).await?;
            }
        }
    }

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

pub async fn insert_option(
    tx: &mut DatabaseTx<'_>,
    coin_state: CoinState,
    lineage_proof: Option<LineageProof>,
    info: OptionInfo,
    context: OptionContext,
) -> Result<bool, WalletError> {
    let coin_id = coin_state.coin.coin_id();

    if let Some(lineage_proof) = lineage_proof {
        tx.insert_lineage_proof(coin_id, lineage_proof).await?;
    }

    let (strike_asset_hash, strike_amount) = match context.metadata.strike_type {
        OptionType::Xch { amount } => (Bytes32::default(), amount),
        OptionType::Cat { asset_id, amount }
        | OptionType::RevocableCat {
            asset_id, amount, ..
        } => {
            tx.insert_asset(Asset {
                hash: asset_id,
                name: None,
                ticker: None,
                precision: 3,
                icon_url: None,
                description: None,
                is_sensitive_content: false,
                is_visible: true,
                hidden_puzzle_hash: None,
                kind: AssetKind::Token,
            })
            .await?;

            (asset_id, amount)
        }
        OptionType::Nft { .. } => {
            warn!("Received an option contract coin with an unsupported strike type, deleting it");
            tx.delete_coin(coin_id).await?;
            return Ok(false);
        }
    };

    let underlying = OptionUnderlying::new(
        info.launcher_id,
        context.creator_puzzle_hash,
        context.metadata.expiration_seconds,
        context.underlying.coin.amount,
        context.metadata.strike_type,
    );

    let coin_info = OptionCoinInfo {
        underlying_coin_hash: info.underlying_coin_id,
        underlying_delegated_puzzle_hash: underlying.delegated_puzzle().tree_hash().into(),
        strike_asset_hash,
        strike_amount,
    };

    let mut asset = Asset {
        hash: info.launcher_id,
        name: None,
        ticker: None,
        precision: 1,
        icon_url: None,
        description: None,
        is_sensitive_content: false,
        is_visible: true,
        hidden_puzzle_hash: None,
        kind: AssetKind::Option,
    };

    tx.insert_asset(asset.clone()).await?;

    // We need to insert the underlying coin first so we can insert the option row
    if let Some(height) = context.underlying.created_height {
        tx.insert_height(height).await?;
    }

    if let Some(height) = context.underlying.spent_height {
        tx.insert_height(height).await?;
    }

    tx.insert_coin(context.underlying).await?;

    // We will never update the option contract row since it's static
    tx.insert_option(info.launcher_id, &coin_info).await?;

    if lineage_proof.is_some() {
        tx.update_coin(
            coin_state.coin.coin_id(),
            info.launcher_id,
            info.p2_puzzle_hash,
        )
        .await?;
    }

    // Now we can insert the option underlying p2 puzzle and asset
    tx.insert_option_p2_puzzle(underlying).await?;

    // TODO: Is it okay to recursively call insert here? We don't allow nested options, so I think so for now.
    let is_underlying_inserted = Box::pin(insert_puzzle(
        tx,
        context.underlying,
        context.underlying_kind.clone(),
        PuzzleContext::None,
        Some(underlying.tree_hash().into()),
    ))
    .await?;

    if !is_underlying_inserted {
        warn!("Failed to insert underlying coin {coin_id}, deleting the option contract coin");
        tx.delete_coin(coin_id).await?;
        return Ok(false);
    }

    let underlying_asset_hash = match &context.underlying_kind {
        ChildKind::Cat { info, .. } => info.asset_id,
        ChildKind::Unknown => Bytes32::default(),
        _ => return Ok(true),
    };

    let underlying_asset = tx.asset(underlying_asset_hash).await?;
    let strike_asset = tx.asset(strike_asset_hash).await?;

    let underlying_ticker = underlying_asset
        .and_then(|asset| asset.ticker)
        .unwrap_or("Unknown".to_string());

    let strike_ticker = strike_asset
        .and_then(|asset| asset.ticker)
        .unwrap_or("Unknown".to_string());

    asset.name = Some(format!("{underlying_ticker} / {strike_ticker}"));

    tx.insert_asset(asset).await?;

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
    let mut cached_coin_states = HashMap::new();

    for input in &transaction.inputs {
        coin_spends.insert(input.coin_spend.coin.coin_id(), input.coin_spend.clone());

        for output in &input.outputs {
            output_coin_ids.insert(output.coin.coin_id());
            cached_coin_states.insert(
                output.coin.coin_id(),
                CoinState::new(output.coin, None, None),
            );
        }
    }

    let peer = peer.with_pending(cached_coin_states, coin_spends.clone());

    let mut puzzle_contexts = HashMap::new();

    for input in &transaction.inputs {
        for output in &input.outputs {
            let context = PuzzleContext::fetch(&peer, genesis_challenge, &output.kind).await?;
            puzzle_contexts.insert(output.coin.coin_id(), context);
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

                tx.update_coin(coin_id, Bytes32::default(), output.coin.puzzle_hash)
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

            // Do the busy work of inserting the asset information into the database now that the coin exists.

            let is_inserted = validate_wallet_coin(&mut tx, coin_id, &output.kind).await?
                && insert_puzzle(
                    &mut tx,
                    coin_state,
                    output.kind.clone(),
                    puzzle_contexts
                        .get(&output.coin.coin_id())
                        .cloned()
                        .unwrap_or_default(),
                    None,
                )
                .await?;

            if is_inserted {
                // We should subscribe to the coin so we know when it's created on-chain.
                if output.kind.subscribe() {
                    subscriptions.push(coin_id);
                }

                if let Some(PuzzleContext::Option(context)) = puzzle_contexts.get(&coin_id) {
                    subscriptions.push(context.underlying.coin.coin_id());
                }
            }
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
