use sage_api::{
    MakeOffer, MakeOfferResponse, TakeOffer, TakeOfferResponse, ViewOffer, ViewOfferResponse,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn make_offer(state: State<'_, AppState>, req: MakeOffer) -> Result<MakeOfferResponse> {
    Ok(state.lock().await.make_offer(req).await?)
}

#[command]
#[specta]
pub async fn take_offer(state: State<'_, AppState>, req: TakeOffer) -> Result<TakeOfferResponse> {
    Ok(state.lock().await.take_offer(req).await?)
}

#[command]
#[specta]
pub async fn view_offer(state: State<'_, AppState>, req: ViewOffer) -> Result<ViewOfferResponse> {
    Ok(state.lock().await.view_offer(req).await?)
}
