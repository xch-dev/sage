use chia::{
    bls::Signature,
    protocol::{Bytes32, CoinState},
};
use sage_database::{DatabaseTx, NftRow};

use crate::{ChildKind, Transaction, WalletError};

use super::compute_nft_info;

pub async fn insert_transaction(
    tx: &mut DatabaseTx<'_>,
    transaction_id: Bytes32,
    transaction: Transaction,
    aggregated_signature: Signature,
) -> Result<(), WalletError> {
    tx.insert_pending_transaction(transaction_id, aggregated_signature, transaction.fee)
        .await?;

    for input in transaction.inputs {
        tx.insert_transaction_spend(
            input.coin_spend.coin,
            transaction_id,
            input.coin_spend.puzzle_reveal,
            input.coin_spend.solution,
        )
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

            match output.kind {
                ChildKind::Launcher => {}
                ChildKind::Cat {
                    asset_id,
                    lineage_proof,
                    p2_puzzle_hash,
                } => {
                    if tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                        tx.insert_coin_state(coin_state, true, Some(transaction_id))
                            .await?;
                        tx.sync_coin(coin_id, Some(p2_puzzle_hash)).await?;
                        tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                            .await?;
                    }
                }
                ChildKind::Did {
                    lineage_proof,
                    info,
                } => {
                    if tx.is_p2_puzzle_hash(info.p2_puzzle_hash).await? {
                        tx.insert_coin_state(coin_state, true, Some(transaction_id))
                            .await?;
                        tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;
                        tx.insert_new_did(info.launcher_id, None, true).await?;
                        tx.insert_did_coin(coin_id, lineage_proof, info).await?;
                    }
                }
                ChildKind::Nft {
                    lineage_proof,
                    info,
                    metadata,
                } => {
                    let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
                    let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
                    let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

                    if tx.is_p2_puzzle_hash(info.p2_puzzle_hash).await? {
                        tx.insert_coin_state(coin_state, true, Some(transaction_id))
                            .await?;

                        tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;

                        let mut row = tx.nft_row(info.launcher_id).await?.unwrap_or(NftRow {
                            launcher_id: info.launcher_id,
                            collection_id: None,
                            minter_did: None,
                            owner_did: info.current_owner,
                            visible: true,
                            name: None,
                            created_height: None,
                            metadata_hash,
                        });

                        if let Some(metadata_hash) = metadata_hash {
                            let blob = tx
                                .fetch_nft_data(metadata_hash)
                                .await?
                                .map(|data| data.blob);

                            // TODO: Handle Minter DID for pending transactions.
                            let info = compute_nft_info(None, blob.as_deref());

                            row.name = info.name;
                        }

                        row.owner_did = info.current_owner;
                        row.created_height = None;

                        tx.insert_nft(row).await?;

                        tx.insert_nft_coin(
                            coin_id,
                            lineage_proof,
                            info,
                            data_hash,
                            metadata_hash,
                            license_hash,
                        )
                        .await?;

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
                ChildKind::Unknown { hint } => {
                    let Some(p2_puzzle_hash) = hint else {
                        continue;
                    };

                    if tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                        tx.insert_coin_state(coin_state, true, Some(transaction_id))
                            .await?;
                        tx.sync_coin(coin_id, hint).await?;
                        tx.insert_unknown_coin(coin_id).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
