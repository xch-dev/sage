use base64::prelude::*;
use bigdecimal::BigDecimal;
use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::Bytes32,
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{decode_address, encode_address};
use clvmr::Allocator;
use hex_literal::hex;
use sage_api::{
    Amount, CatRecord, CoinRecord, DidRecord, GetCollectionNfts, GetNftCollections, GetNfts,
    NftCollectionRecord, NftInfo, NftRecord, NftSortMode, NftStatus, PendingTransactionRecord,
    SyncStatus,
};
use sage_database::{DidRow, NftData, NftRow};
use sage_wallet::WalletError;
use specta::specta;
use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
};

#[command]
#[specta]
pub async fn get_addresses(state: State<'_, AppState>) -> Result<Vec<String>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let puzzle_hashes = wallet.db.p2_puzzle_hashes_unhardened().await?;
    let addresses = puzzle_hashes
        .into_iter()
        .map(|puzzle_hash| {
            Ok(encode_address(
                puzzle_hash.to_bytes(),
                &state.network().address_prefix,
            )?)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(addresses)
}

#[command]
#[specta]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut tx = wallet.db.tx().await?;
    let balance = tx.balance().await?;
    let total_coins = tx.total_coin_count().await?;
    let synced_coins = tx.synced_coin_count().await?;
    tx.commit().await?;

    let puzzle_hash = match wallet.p2_puzzle_hash(false, false).await {
        Ok(puzzle_hash) => Some(puzzle_hash),
        Err(WalletError::InsufficientDerivations) => None,
        Err(error) => return Err(error.into()),
    };

    let receive_address = puzzle_hash
        .map(|puzzle_hash| encode_address(puzzle_hash.to_bytes(), &state.network().address_prefix))
        .transpose()?;

    Ok(SyncStatus {
        balance: Amount::from_mojos(balance, state.unit.decimals),
        unit: state.unit.clone(),
        total_coins,
        synced_coins,
        receive_address: receive_address.unwrap_or_default(),
        burn_address: encode_address(
            hex!("000000000000000000000000000000000000000000000000000000000000dead"),
            &state.network().address_prefix,
        )?,
    })
}

#[command]
#[specta]
pub async fn get_coins(state: State<'_, AppState>) -> Result<Vec<CoinRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut tx = wallet.db.tx().await?;
    let mut records = Vec::new();

    let rows = tx.p2_coin_states().await?;

    for row in rows {
        let cs = row.coin_state;

        let spend_transaction_id = tx
            .transactions_for_coin(cs.coin.coin_id())
            .await?
            .into_iter()
            .map(hex::encode)
            .next();

        records.push(CoinRecord {
            coin_id: hex::encode(cs.coin.coin_id()),
            address: encode_address(
                cs.coin.puzzle_hash.to_bytes(),
                &state.network().address_prefix,
            )?,
            amount: Amount::from_mojos(cs.coin.amount as u128, state.unit.decimals),
            created_height: cs.created_height,
            spent_height: cs.spent_height,
            create_transaction_id: row.transaction_id.map(hex::encode),
            spend_transaction_id,
        });
    }

    Ok(records)
}

#[command]
#[specta]
pub async fn get_cat_coins(
    state: State<'_, AppState>,
    asset_id: String,
) -> Result<Vec<CoinRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    let mut tx = wallet.db.tx().await?;
    let mut records = Vec::new();

    let rows = tx.cat_coin_states(asset_id.into()).await?;

    for row in rows {
        let cs = row.coin_state;

        let spend_transaction_id = tx
            .transactions_for_coin(cs.coin.coin_id())
            .await?
            .into_iter()
            .map(hex::encode)
            .next();

        records.push(CoinRecord {
            coin_id: hex::encode(cs.coin.coin_id()),
            address: encode_address(
                cs.coin.puzzle_hash.to_bytes(),
                &state.network().address_prefix,
            )?,
            amount: Amount::from_mojos(cs.coin.amount as u128, 3),
            created_height: cs.created_height,
            spent_height: cs.spent_height,
            create_transaction_id: row.transaction_id.map(hex::encode),
            spend_transaction_id,
        });
    }

    Ok(records)
}

#[command]
#[specta]
pub async fn get_cats(state: State<'_, AppState>) -> Result<Vec<CatRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let cats = wallet.db.cats().await?;

    let mut records = Vec::with_capacity(cats.len());

    for cat in cats {
        let balance = wallet.db.cat_balance(cat.asset_id).await?;

        records.push(CatRecord {
            asset_id: hex::encode(cat.asset_id),
            name: cat.name,
            ticker: cat.ticker,
            description: cat.description,
            icon_url: cat.icon,
            visible: cat.visible,
            balance: Amount::from_mojos(balance, 3),
        });
    }

    Ok(records)
}

#[command]
#[specta]
pub async fn get_cat(state: State<'_, AppState>, asset_id: String) -> Result<Option<CatRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    let cat = wallet.db.cat(asset_id.into()).await?;
    let balance = wallet.db.cat_balance(asset_id.into()).await?;

    cat.map(|cat| {
        Ok(CatRecord {
            asset_id: hex::encode(cat.asset_id),
            name: cat.name,
            ticker: cat.ticker,
            description: cat.description,
            icon_url: cat.icon,
            visible: cat.visible,
            balance: Amount::from_mojos(balance, 3),
        })
    })
    .transpose()
}

#[command]
#[specta]
pub async fn get_dids(state: State<'_, AppState>) -> Result<Vec<DidRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .did_coins()
        .await?
        .into_iter()
        .map(
            |DidRow {
                 did,
                 name,
                 visible,
                 created_height,
                 create_transaction_id,
             }| {
                Ok(DidRecord {
                    launcher_id: encode_address(did.info.launcher_id.to_bytes(), "did:chia:")?,
                    name,
                    visible,
                    coin_id: hex::encode(did.coin.coin_id()),
                    address: encode_address(
                        did.info.p2_puzzle_hash.to_bytes(),
                        &state.network().address_prefix,
                    )?,
                    amount: Amount::from_mojos(did.coin.amount as u128, state.unit.decimals),
                    created_height,
                    create_transaction_id: create_transaction_id.map(hex::encode),
                })
            },
        )
        .collect()
}

#[command]
#[specta]
pub async fn get_pending_transactions(
    state: State<'_, AppState>,
) -> Result<Vec<PendingTransactionRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .transactions()
        .await?
        .into_iter()
        .map(|tx| {
            Ok(PendingTransactionRecord {
                transaction_id: hex::encode(tx.transaction_id),
                fee: Amount::from_mojos(tx.fee as u128, state.unit.decimals),
                // TODO: Date format?
                submitted_at: tx.submitted_at.map(|ts| ts.to_string()),
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_nft_status(state: State<'_, AppState>) -> Result<NftStatus> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut tx = wallet.db.tx().await?;

    let nfts = tx.nft_count().await?;
    let collections = tx.collection_count().await?;
    let visible_nfts = tx.visible_nft_count().await?;
    let visible_collections = tx.visible_collection_count().await?;

    tx.commit().await?;

    Ok(NftStatus {
        nfts,
        visible_nfts,
        collections,
        visible_collections,
    })
}

#[command]
#[specta]
pub async fn get_nft_collections(
    state: State<'_, AppState>,
    request: GetNftCollections,
) -> Result<Vec<NftCollectionRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut records = Vec::new();

    let mut tx = wallet.db.tx().await?;

    let collections = if request.include_hidden {
        tx.collections_named(request.limit, request.offset).await?
    } else {
        tx.collections_visible_named(request.limit, request.offset)
            .await?
    };

    for col in collections {
        let total = tx.collection_nft_count(col.collection_id).await?;
        let total_visible = tx.collection_visible_nft_count(col.collection_id).await?;

        records.push(NftCollectionRecord {
            collection_id: encode_address(col.collection_id.to_bytes(), "col")?,
            did_id: encode_address(col.did_id.to_bytes(), "did:chia:")?,
            metadata_collection_id: col.metadata_collection_id,
            visible: col.visible,
            name: col.name,
            icon: col.icon,
            nfts: total,
            visible_nfts: total_visible,
        });
    }

    tx.commit().await?;

    Ok(records)
}

#[command]
#[specta]
pub async fn get_nft_collection(
    state: State<'_, AppState>,
    collection_id: Option<String>,
) -> Result<NftCollectionRecord> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let collection_id = if let Some(collection_id) = collection_id {
        let (collection_id, prefix) = decode_address(&collection_id)?;

        if prefix != "col" {
            return Err(Error::invalid_prefix(&prefix));
        }

        Some(Bytes32::from(collection_id))
    } else {
        None
    };

    let collection = if let Some(collection_id) = collection_id {
        Some(wallet.db.collection(collection_id).await?)
    } else {
        None
    };

    let mut tx = wallet.db.tx().await?;

    let total = if let Some(collection_id) = collection_id {
        tx.collection_nft_count(collection_id).await?
    } else {
        tx.no_collection_nft_count().await?
    };

    let total_visible = if let Some(collection_id) = collection_id {
        tx.collection_visible_nft_count(collection_id).await?
    } else {
        tx.no_collection_visible_nft_count().await?
    };

    tx.commit().await?;

    Ok(if let Some(collection) = collection {
        NftCollectionRecord {
            collection_id: encode_address(collection.collection_id.to_bytes(), "col")?,
            did_id: encode_address(collection.did_id.to_bytes(), "did:chia:")?,
            metadata_collection_id: collection.metadata_collection_id,
            visible: collection.visible,
            name: collection.name,
            icon: collection.icon,
            nfts: total,
            visible_nfts: total_visible,
        }
    } else {
        NftCollectionRecord {
            collection_id: "None".to_string(),
            did_id: "Miscellaneous".to_string(),
            metadata_collection_id: "None".to_string(),
            visible: true,
            name: Some("Uncategorized".to_string()),
            icon: None,
            nfts: total,
            visible_nfts: total_visible,
        }
    })
}

#[command]
#[specta]
pub async fn get_nfts(state: State<'_, AppState>, request: GetNfts) -> Result<Vec<NftRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut records = Vec::new();

    let mut tx = wallet.db.tx().await?;

    let nfts = match (request.sort_mode, request.include_hidden) {
        (NftSortMode::Name, true) => tx.nfts_named(request.limit, request.offset).await?,
        (NftSortMode::Name, false) => tx.nfts_visible_named(request.limit, request.offset).await?,
        (NftSortMode::Recent, true) => tx.nfts_recent(request.limit, request.offset).await?,
        (NftSortMode::Recent, false) => {
            tx.nfts_visible_recent(request.limit, request.offset)
                .await?
        }
    };

    for nft in nfts {
        let data = if let Some(hash) = tx.data_hash(nft.launcher_id).await? {
            tx.fetch_nft_data(hash).await?
        } else {
            None
        };

        let collection_name = if let Some(collection_id) = nft.collection_id {
            tx.collection_name(collection_id).await?
        } else {
            None
        };

        records.push(nft_record(nft, collection_name, data)?);
    }

    tx.commit().await?;

    Ok(records)
}

#[command]
#[specta]
pub async fn get_collection_nfts(
    state: State<'_, AppState>,
    request: GetCollectionNfts,
) -> Result<Vec<NftRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let collection_id = if let Some(collection_id) = request.collection_id {
        let (collection_id, prefix) = decode_address(&collection_id)?;

        if prefix != "col" {
            return Err(Error::invalid_prefix(&prefix));
        }

        Some(Bytes32::from(collection_id))
    } else {
        None
    };

    let mut records = Vec::new();

    let mut tx = wallet.db.tx().await?;

    let nfts = match (request.sort_mode, request.include_hidden, collection_id) {
        (NftSortMode::Name, true, Some(collection_id)) => {
            tx.collection_nfts_named(collection_id, request.limit, request.offset)
                .await?
        }
        (NftSortMode::Name, false, Some(collection_id)) => {
            tx.collection_nfts_visible_named(collection_id, request.limit, request.offset)
                .await?
        }
        (NftSortMode::Recent, true, Some(collection_id)) => {
            tx.collection_nfts_recent(collection_id, request.limit, request.offset)
                .await?
        }
        (NftSortMode::Recent, false, Some(collection_id)) => {
            tx.collection_nfts_visible_recent(collection_id, request.limit, request.offset)
                .await?
        }
        (NftSortMode::Name, true, None) => {
            tx.no_collection_nfts_named(request.limit, request.offset)
                .await?
        }
        (NftSortMode::Name, false, None) => {
            tx.no_collection_nfts_visible_named(request.limit, request.offset)
                .await?
        }
        (NftSortMode::Recent, true, None) => {
            tx.no_collection_nfts_recent(request.limit, request.offset)
                .await?
        }
        (NftSortMode::Recent, false, None) => {
            tx.no_collection_nfts_visible_recent(request.limit, request.offset)
                .await?
        }
    };

    for nft in nfts {
        let data = if let Some(hash) = tx.data_hash(nft.launcher_id).await? {
            tx.fetch_nft_data(hash).await?
        } else {
            None
        };

        let collection_name = if let Some(collection_id) = nft.collection_id {
            tx.collection_name(collection_id).await?
        } else {
            None
        };

        records.push(nft_record(nft, collection_name, data)?);
    }

    tx.commit().await?;

    Ok(records)
}

#[command]
#[specta]
pub async fn get_nft(state: State<'_, AppState>, launcher_id: String) -> Result<Option<NftInfo>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (launcher_id, prefix) = decode_address(&launcher_id)?;
    if prefix != "nft" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let mut tx = wallet.db.tx().await?;

    let Some(nft_row) = tx.nft_row(launcher_id.into()).await? else {
        return Ok(None);
    };

    let Some(nft) = tx.nft(launcher_id.into()).await? else {
        return Ok(None);
    };

    let mut allocator = Allocator::new();
    let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
    let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

    let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
    let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
    let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

    let data = if let Some(hash) = data_hash {
        tx.fetch_nft_data(hash).await?
    } else {
        None
    };

    let offchain_metadata = if let Some(hash) = metadata_hash {
        tx.fetch_nft_data(hash).await?
    } else {
        None
    };

    let collection_name = if let Some(collection_id) = nft_row.collection_id {
        tx.collection_name(collection_id).await?
    } else {
        None
    };

    tx.commit().await?;

    Ok(Some(NftInfo {
        launcher_id: encode_address(nft_row.launcher_id.to_bytes(), "nft")?,
        collection_id: nft_row
            .collection_id
            .map(|col| encode_address(col.to_bytes(), "col"))
            .transpose()?,
        collection_name,
        minter_did: nft_row
            .minter_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        owner_did: nft_row
            .owner_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        visible: nft_row.visible,
        coin_id: hex::encode(nft.coin.coin_id()),
        address: encode_address(
            nft.info.p2_puzzle_hash.to_bytes(),
            &state.network().address_prefix,
        )?,
        royalty_address: encode_address(
            nft.info.royalty_puzzle_hash.to_bytes(),
            &state.network().address_prefix,
        )?,
        royalty_percent: (BigDecimal::from(nft.info.royalty_ten_thousandths)
            / BigDecimal::from(100))
        .to_string(),
        data_uris: metadata
            .as_ref()
            .map(|m| m.data_uris.clone())
            .unwrap_or_default(),
        data_hash: data_hash.map(hex::encode),
        metadata_uris: metadata
            .as_ref()
            .map(|m| m.metadata_uris.clone())
            .unwrap_or_default(),
        metadata_hash: metadata_hash.map(hex::encode),
        license_uris: metadata
            .as_ref()
            .map(|m| m.license_uris.clone())
            .unwrap_or_default(),
        license_hash: license_hash.map(hex::encode),
        edition_number: metadata
            .as_ref()
            .map(|m| m.edition_number.try_into())
            .transpose()?,
        edition_total: metadata
            .as_ref()
            .map(|m| m.edition_total.try_into())
            .transpose()?,
        created_height: nft_row.created_height,
        data: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.blob)),
        data_mime_type: data.map(|data| data.mime_type),
        metadata: offchain_metadata.and_then(|offchain_metadata| {
            if offchain_metadata.mime_type == "application/json" {
                String::from_utf8(offchain_metadata.blob).ok()
            } else {
                None
            }
        }),
    }))
}

fn nft_record(
    nft: NftRow,
    collection_name: Option<String>,
    data: Option<NftData>,
) -> Result<NftRecord> {
    Ok(NftRecord {
        launcher_id: encode_address(nft.launcher_id.to_bytes(), "nft")?,
        collection_id: nft
            .collection_id
            .map(|col| encode_address(col.to_bytes(), "col"))
            .transpose()?,
        collection_name,
        minter_did: nft
            .minter_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        owner_did: nft
            .owner_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        visible: nft.visible,
        sensitive_content: nft.sensitive_content,
        name: nft.name,
        data: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.blob)),
        data_mime_type: data.map(|data| data.mime_type),
        created_height: nft.created_height,
    })
}
