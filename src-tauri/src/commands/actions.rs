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

    wallet.db.delete_cat(asset_id.into()).await?;

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
            precision: record.precision,
            icon_url: record.icon_url,
        })
        .await?;

    Ok(())
}
