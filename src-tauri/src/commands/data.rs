use base64::prelude::*;
use bigdecimal::BigDecimal;
use chia::{
    clvm_traits::{FromClvm, ToClvm},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::encode_address;
use clvmr::Allocator;
use sage_api::{
    Amount, CatRecord, CoinRecord, DidRecord, GetNfts, GetNftsResponse, NftRecord, SyncStatus,
};
use sage_database::{NftData, NftDisplayInfo};
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
    let balance = tx.p2_balance().await?;
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
    })
}

#[command]
#[specta]
pub async fn get_coins(state: State<'_, AppState>) -> Result<Vec<CoinRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let coin_states = wallet.db.p2_coin_states().await?;

    coin_states
        .into_iter()
        .map(|cs| {
            Ok(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(
                    cs.coin.puzzle_hash.to_bytes(),
                    &state.network().address_prefix,
                )?,
                amount: Amount::from_mojos(cs.coin.amount as u128, state.unit.decimals),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
            })
        })
        .collect()
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

    let coin_states = wallet.db.cat_coin_states(asset_id.into()).await?;

    coin_states
        .into_iter()
        .map(|cs| {
            Ok(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(
                    cs.coin.puzzle_hash.to_bytes(),
                    &state.network().address_prefix,
                )?,
                amount: Amount::from_mojos(cs.coin.amount as u128, 3),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_cats(state: State<'_, AppState>) -> Result<Vec<CatRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let cats = wallet.db.cats().await?;

    cats.into_iter()
        .map(|cat| {
            Ok(CatRecord {
                asset_id: hex::encode(cat.asset_id),
                name: cat.name,
                ticker: cat.ticker,
                description: cat.description,
                icon_url: cat.icon_url,
                visible: cat.visible,
            })
        })
        .collect()
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
        .map(|did| {
            Ok(DidRecord {
                encoded_id: encode_address(did.info.launcher_id.to_bytes(), "did:chia:")?,
                launcher_id: hex::encode(did.info.launcher_id),
                coin_id: hex::encode(did.coin.coin_id()),
                address: encode_address(
                    did.info.p2_puzzle_hash.to_bytes(),
                    &state.network().address_prefix,
                )?,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_nfts(state: State<'_, AppState>, request: GetNfts) -> Result<GetNftsResponse> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut records = Vec::new();

    let mut tx = wallet.db.tx().await?;

    for nft in tx.fetch_nfts(request.limit, request.offset).await? {
        let data = if let Some(hash) = nft.data_hash {
            tx.fetch_nft_data(hash).await?
        } else {
            None
        };

        let metadata = if let Some(hash) = nft.metadata_hash {
            tx.fetch_nft_data(hash).await?
        } else {
            None
        };

        records.push(nft_record(
            &nft,
            &state.network().address_prefix,
            data,
            metadata,
        )?);
    }

    let total = tx.nft_count().await?;

    tx.commit().await?;

    Ok(GetNftsResponse {
        items: records,
        total,
    })
}

#[command]
#[specta]
pub async fn get_nft(state: State<'_, AppState>, launcher_id: String) -> Result<Option<NftRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let launcher_id: [u8; 32] = hex::decode(launcher_id)?
        .try_into()
        .map_err(|_| Error::invalid_launcher_id())?;

    let mut tx = wallet.db.tx().await?;

    let Some(nft) = tx.fetch_nft(launcher_id.into()).await? else {
        return Ok(None);
    };

    let data = if let Some(hash) = nft.data_hash {
        tx.fetch_nft_data(hash).await?
    } else {
        None
    };

    let metadata = if let Some(hash) = nft.metadata_hash {
        tx.fetch_nft_data(hash).await?
    } else {
        None
    };

    tx.commit().await?;

    let record = nft_record(&nft, &state.network().address_prefix, data, metadata)?;

    Ok(Some(record))
}

fn nft_record(
    nft: &NftDisplayInfo,
    prefix: &str,
    data: Option<NftData>,
    offchain_metadata: Option<NftData>,
) -> Result<NftRecord> {
    let mut allocator = Allocator::new();
    let ptr = nft.info.metadata.to_clvm(&mut allocator)?;
    let metadata = NftMetadata::from_clvm(&allocator, ptr).ok();

    Ok(NftRecord {
        launcher_id_hex: hex::encode(nft.info.launcher_id),
        launcher_id: encode_address(nft.info.launcher_id.to_bytes(), "nft")?,
        owner_did: nft
            .info
            .current_owner
            .map(|owner| encode_address(owner.to_bytes(), "did:chia:"))
            .transpose()?,
        coin_id: hex::encode(nft.coin_id),
        address: encode_address(nft.info.p2_puzzle_hash.to_bytes(), prefix)?,
        royalty_address: encode_address(nft.info.royalty_puzzle_hash.to_bytes(), prefix)?,
        royalty_percent: (BigDecimal::from(nft.info.royalty_ten_thousandths)
            / BigDecimal::from(100))
        .to_string(),
        data_uris: metadata
            .as_ref()
            .map(|m| m.data_uris.clone())
            .unwrap_or_default(),
        data_hash: nft.data_hash.map(hex::encode),
        metadata_uris: metadata
            .as_ref()
            .map(|m| m.metadata_uris.clone())
            .unwrap_or_default(),
        metadata_hash: nft.metadata_hash.map(hex::encode),
        license_uris: metadata
            .as_ref()
            .map(|m| m.license_uris.clone())
            .unwrap_or_default(),
        license_hash: nft.license_hash.map(hex::encode),
        edition_number: metadata
            .as_ref()
            .map(|m| m.edition_number.try_into())
            .transpose()?,
        edition_total: metadata
            .as_ref()
            .map(|m| m.edition_total.try_into())
            .transpose()?,
        data_mime_type: data.as_ref().map(|data| data.mime_type.clone()),
        data: data.map(|data| BASE64_STANDARD.encode(&data.blob)),
        metadata: offchain_metadata.and_then(|offchain_metadata| {
            if offchain_metadata.mime_type == "application/json" {
                String::from_utf8(offchain_metadata.blob).ok()
            } else {
                None
            }
        }),
    })
}
