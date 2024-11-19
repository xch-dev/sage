use sage_api::CatRecord;
use sage_database::{CatRow, DidRow};
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

    let asset_id = parse_asset_id(asset_id)?;
    wallet.db.refetch_cat(asset_id).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_cat_info(state: State<'_, AppState>, record: CatRecord) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id = parse_asset_id(record.asset_id)?;

    wallet
        .db
        .update_cat(CatRow {
            asset_id,
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

    let did_id = parse_did_id(did_id)?;

    let Some(row) = wallet.db.did_row(did_id).await? else {
        return Err(Error {
            kind: ErrorKind::NotFound,
            reason: "DID not found".into(),
        });
    };

    wallet
        .db
        .insert_did(DidRow {
            launcher_id: row.launcher_id,
            coin_id: row.coin_id,
            name,
            is_owned: row.is_owned,
            visible,
            created_height: row.created_height,
        })
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_nft(state: State<'_, AppState>, nft_id: String, visible: bool) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let nft_id = parse_nft_id(nft_id)?;
    wallet.db.set_nft_visible(nft_id, visible).await?;

    Ok(())
}
