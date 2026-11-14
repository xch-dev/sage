use tauri::State;

use crate::{AppState, SharedSageApp};

pub(crate) async fn ensure_app_is_enabled_for_scope(
    app_state: &State<'_, AppState>,
    app: &SharedSageApp,
) -> Result<(), String> {
    if app_is_enabled_for_scope(app_state, app).await {
        return Ok(());
    }

    Err("App is not enabled for the current scope".into())
}

async fn app_is_enabled_for_scope(app_state: &State<'_, AppState>, app: &SharedSageApp) -> bool {
    let Some(fingerprint) = current_wallet_fingerprint(app_state).await else {
        return false;
    };

    app.with(|sage_app| sage_app.common().wallet_scope().allows(fingerprint))
}

pub(crate) async fn current_wallet_fingerprint(app_state: &State<'_, AppState>) -> Option<u32> {
    let state = app_state.lock().await;
    state.config.global.fingerprint
}
