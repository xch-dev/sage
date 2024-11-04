use chia_wallet_sdk::decode_address;
use sage_api::CatRecord;
use sage_database::CatRow;
use specta::specta;
use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
};

#[command]
#[specta]
pub async fn remove_cat_info(state: State<'_, AppState>, asset_id: String) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    wallet.db.refetch_cat(asset_id.into()).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_cat_info(state: State<'_, AppState>, record: CatRecord) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(record.asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    wallet
        .db
        .update_cat(CatRow {
            asset_id: asset_id.into(),
            name: record.name,
            description: record.description,
            ticker: record.ticker,
            icon: record.icon_url,
            visible: record.visible,
            fetched: true,
        })
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_did(
    state: State<'_, AppState>,
    did_id: String,
    name: Option<String>,
    visible: bool,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (launcher_id, prefix) = decode_address(&did_id)?;

    if prefix != "did:chia:" {
        return Err(Error::invalid_prefix(&prefix));
    }

    wallet
        .db
        .update_did(launcher_id.into(), name, visible)
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_nft(state: State<'_, AppState>, nft_id: String, visible: bool) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (launcher_id, prefix) = decode_address(&nft_id)?;

    if prefix != "nft" {
        return Err(Error::invalid_prefix(&prefix));
    }

    wallet
        .db
        .set_nft_visible(launcher_id.into(), visible)
        .await?;

    Ok(())
}
