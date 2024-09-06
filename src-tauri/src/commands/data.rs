use bigdecimal::BigDecimal;
use chia_wallet_sdk::encode_address;
use sage_api::{
    Amount, CatRecord, CoinRecord, DidRecord, GetNfts, GetNftsResponse, NftRecord, SyncStatus,
};
use sage_database::{NftRow, NftUri, NftUriKind};
use specta::specta;
use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
    utils::fetch_puzzle_hash,
};

#[command]
#[specta]
pub async fn get_addresses(state: State<'_, AppState>) -> Result<Vec<String>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let puzzle_hashes = wallet.db.p2_puzzle_hashes_unhardened().await?;
    let addresses = puzzle_hashes
        .into_iter()
        .map(|puzzle_hash| Ok(encode_address(puzzle_hash.to_bytes(), state.prefix())?))
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

    let puzzle_hash = fetch_puzzle_hash(&mut tx).await?;

    let receive_address = puzzle_hash
        .map(|puzzle_hash| encode_address(puzzle_hash.to_bytes(), state.prefix()))
        .transpose()?;

    tx.commit().await?;

    Ok(SyncStatus {
        balance: Amount::from_mojos(balance, state.unit().decimals),
        unit: state.unit().clone(),
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
                address: encode_address(cs.coin.puzzle_hash.to_bytes(), state.prefix())?,
                amount: Amount::from_mojos(cs.coin.amount as u128, state.unit().decimals),
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
                address: encode_address(cs.coin.puzzle_hash.to_bytes(), state.prefix())?,
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
                description: cat.description,
                ticker: cat.ticker,
                precision: cat.precision,
                icon_url: cat.icon_url,
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
                address: encode_address(did.info.p2_puzzle_hash.to_bytes(), state.prefix())?,
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

    for nft in tx.nfts(request.limit, request.offset).await? {
        let uris = tx.nft_uris(nft.launcher_id).await?;
        records.push(nft_record(nft, uris, state.prefix())?);
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

    let Some(nft) = tx.nft(launcher_id.into()).await? else {
        return Ok(None);
    };
    let uris = tx.nft_uris(nft.launcher_id).await?;
    let record = nft_record(nft, uris, state.prefix())?;

    tx.commit().await?;

    Ok(Some(record))
}

fn nft_record(nft: NftRow, uris: Vec<NftUri>, prefix: &str) -> Result<NftRecord> {
    let mut data_uris = Vec::new();
    let mut metadata_uris = Vec::new();
    let mut license_uris = Vec::new();

    for uri in uris {
        match uri.kind {
            NftUriKind::Data => data_uris.push(uri.uri),
            NftUriKind::Metadata => metadata_uris.push(uri.uri),
            NftUriKind::License => license_uris.push(uri.uri),
        }
    }

    Ok(NftRecord {
        encoded_id: encode_address(nft.launcher_id.to_bytes(), "nft")?,
        launcher_id: hex::encode(nft.launcher_id),
        encoded_owner_did: nft
            .current_owner
            .map(|owner| encode_address(owner.to_bytes(), "did:chia:"))
            .transpose()?,
        owner_did: nft.current_owner.map(hex::encode),
        coin_id: hex::encode(nft.coin_id),
        address: encode_address(nft.p2_puzzle_hash.to_bytes(), prefix)?,
        royalty_address: encode_address(nft.royalty_puzzle_hash.to_bytes(), prefix)?,
        royalty_percent: (BigDecimal::from(nft.royalty_ten_thousandths) / BigDecimal::from(100))
            .to_string(),
        data_uris,
        data_hash: nft.data_hash.map(hex::encode),
        metadata_uris,
        metadata_json: nft.metadata_json,
        metadata_hash: nft.metadata_hash.map(hex::encode),
        license_uris,
        license_hash: nft.license_hash.map(hex::encode),
        edition_number: nft.edition_number,
        edition_total: nft.edition_total,
    })
}
