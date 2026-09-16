use tauri::{AppHandle, State, command};

use crate::{
    AppsHostState, Result, SageAppUrlPreview, SageAppView, apply_app_update_inner,
    check_app_update_inner,
};

#[command]
#[specta::specta]
pub async fn apps_check_app_update(
    apps_state: State<'_, AppsHostState>,
    app_handle: AppHandle,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    check_app_update_inner(&app_handle, &apps_state, &app_id).await
}

#[command]
#[specta::specta]
pub async fn apps_apply_app_update(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
    expected_manifest_hash: String,
) -> Result<SageAppView> {
    let expected_manifest_hash = Some(expected_manifest_hash.as_str());

    apply_app_update_inner(
        &app_handle,
        &apps_state,
        &app_id,
        None,
        expected_manifest_hash,
    )
    .await
}
