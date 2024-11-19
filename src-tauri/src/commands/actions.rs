use sage_api::{
    RemoveCat, RemoveCatResponse, UpdateCat, UpdateCatResponse, UpdateDid, UpdateDidResponse,
    UpdateNft, UpdateNftResponse,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn remove_cat(state: State<'_, AppState>, req: RemoveCat) -> Result<RemoveCatResponse> {
    Ok(state.lock().await.remove_cat(req).await?)
}

#[command]
#[specta]
pub async fn update_cat(state: State<'_, AppState>, req: UpdateCat) -> Result<UpdateCatResponse> {
    Ok(state.lock().await.update_cat(req).await?)
}

#[command]
#[specta]
pub async fn update_did(state: State<'_, AppState>, req: UpdateDid) -> Result<UpdateDidResponse> {
    Ok(state.lock().await.update_did(req).await?)
}

#[command]
#[specta]
pub async fn update_nft(state: State<'_, AppState>, req: UpdateNft) -> Result<UpdateNftResponse> {
    Ok(state.lock().await.update_nft(req).await?)
}
