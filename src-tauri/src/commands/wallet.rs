use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
    models::DerivationInfo,
};

#[command]
pub async fn derivation_info(state: State<'_, AppState>) -> Result<DerivationInfo> {
    let state = state.lock().await;
    let _active = state.wallet.as_ref().ok_or(Error::NoActiveWallet)?;
    todo!()
}
