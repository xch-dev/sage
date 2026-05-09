use tauri::{command, AppHandle, State};

use crate::host::Result;
use crate::lifecycle::update::logic::{check_app_update_inner, apply_app_update_inner};
use crate::types::{
    SageAppUrlPreview, SageAppView,
};
use crate::AppsHostState;

#[command]
#[specta::specta]
pub async fn check_app_update(
    apps_state: State<'_, AppsHostState>,
    app_handle: AppHandle,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    check_app_update_inner(&app_handle, &apps_state, &app_id).await
}

#[command]
#[specta::specta]
pub async fn apply_app_update(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
) -> Result<SageAppView> {
    apply_app_update_inner(
        &app_handle,
        &apps_state,
        &app_id,
        None,
    ).await
}
