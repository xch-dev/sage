use std::io;

use tauri::{AppHandle, State, command};

use crate::bridge::USER_BRIDGE_CHANNEL;
use crate::bridge::event_emit::emit_bridge_event_to_app_id;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::host::AppState;
use crate::host::Result;
use crate::lifecycle::update::permissions::update_app_permissions;
use crate::lifecycle::{
    download_url_snapshot, read_installed_app_by_id, write_installed_app_metadata,
};
use crate::types::{
    SageAppUrlPreview, SageGrantedPermissions, UserSageApp, UserSageAppPendingUpdate,
    UserSageAppSource,
};

async fn fetch_pending_update(app: &UserSageApp) -> Result<Option<UserSageAppPendingUpdate>> {
    let app_url = match app.source() {
        UserSageAppSource::Url { app_url } => app_url.clone(),
        UserSageAppSource::Zip => return Ok(None),
    };

    let preview = SageAppUrlPreview::new(&app_url)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    let active_snapshot = app.common().active_snapshot();

    if preview.manifest_hash() == active_snapshot.manifest_hash()
        && preview.manifest() == active_snapshot.manifest()
    {
        return Ok(None);
    }

    Ok(Some(UserSageAppPendingUpdate::new(
        app_url,
        preview.manifest_hash().to_string(),
        preview.manifest().clone(),
    )))
}

#[command]
#[specta::specta]
pub async fn check_app_update(
    state: State<'_, AppState>,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let app = read_installed_app_by_id(&base_path, &app_id)
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let Some(pending) = fetch_pending_update(&app).await? else {
        return Ok(None);
    };

    if let Some(existing_pending) = app.pending_update()
        && existing_pending.manifest_hash() == pending.manifest_hash()
        && existing_pending.manifest() == pending.manifest()
    {
        return Ok(None);
    }

    Ok(Some(SageAppUrlPreview::from_pending_update(&pending)))
}

#[command]
#[specta::specta]
pub async fn download_app_update(
    state: State<'_, AppState>,
    app_id: String,
) -> Result<UserSageApp> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let mut app = read_installed_app_by_id(&base_path, &app_id)
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let Some(pending) = fetch_pending_update(&app).await? else {
        return Ok(app);
    };

    app.set_pending_update(Some(pending));

    write_installed_app_metadata(&app)
        .map_err(|err| io::Error::other(format!("failed to write app metadata: {err}")))?;

    Ok(app)
}

#[command]
#[specta::specta]
pub async fn apply_app_update(
    state: State<'_, AppState>,
    app_id: String,
    granted_permissions: SageGrantedPermissions,
) -> Result<UserSageApp> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let mut app = read_installed_app_by_id(&base_path, &app_id)
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let pending = match app.pending_update().cloned() {
        Some(pending) => pending,
        None => fetch_pending_update(&app)
            .await?
            .ok_or_else(|| io::Error::other(format!("app {app_id} has no available update")))?,
    };

    let snapshot = download_url_snapshot(
        &app.app_path(),
        pending.app_url(),
        pending.manifest(),
        pending.manifest_hash(),
    )
    .await
    .map_err(|err| io::Error::other(format!("failed to download update snapshot: {err}")))?;

    app.common_mut()
        .apply_update(&pending, granted_permissions, snapshot)
        .map_err(|err| {
            io::Error::other(format!("failed to apply app update permissions: {err}"))
        })?;

    app.set_pending_update(None);

    write_installed_app_metadata(&app)
        .map_err(|err| io::Error::other(format!("failed to write app metadata: {err}")))?;

    Ok(app)
}

#[command]
#[specta::specta]
pub async fn apps_update_permissions(
    app: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
    granted_permissions: SageGrantedPermissions,
) -> Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let update_result = update_app_permissions(&base_path, &app_id, &granted_permissions)
        .map_err(|err| io::Error::other(format!("failed to update app permissions: {err}")))?;

    let capability_change = update_result.change().capabilities();
    if !capability_change.added.is_empty() || !capability_change.removed.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            &app,
            &app_id,
            EventForApp::from_capabilities_change(USER_BRIDGE_CHANNEL, capability_change.clone()),
        )
        .await;
    }

    let network_change = update_result.change().network_whitelist();
    if !network_change.added.is_empty() || !network_change.removed.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            &app,
            &app_id,
            EventForApp::from_network_whitelist_change(USER_BRIDGE_CHANNEL, network_change.clone()),
        )
        .await;
    }

    Ok(())
}
