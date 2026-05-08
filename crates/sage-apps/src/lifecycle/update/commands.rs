use std::io;

use tauri::{command, AppHandle, State};

use crate::host::AppState;
use crate::host::Result;
use crate::lifecycle::download_url_snapshot;
use crate::lifecycle::update::logic::{
    check_app_update_for_app, fetch_pending_update,
    fetch_pending_update_for_resolved_stopped_app,
};
use crate::runtime::resolve_app;
use crate::types::{
    SageApp, SageAppUrlPreview, SageAppView, SageGrantedPermissionsInput,
};

#[command]
#[specta::specta]
pub async fn check_app_update(
    app_handle: AppHandle,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    let app = resolve_app(&app_handle, &app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    check_app_update_for_app(&app).await
}

#[command]
#[specta::specta]
pub async fn download_app_update(app_handle: AppHandle, app_id: String) -> Result<SageAppView> {
    let app = resolve_app(&app_handle, &app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let app = app.clone_app_for_operation();

    let Some(pending) = fetch_pending_update(&app).await? else {
        return Ok((&app).into());
    };

    app.try_mutate(|app| {
        app.set_pending_update(Some(pending))
            .map_err(|err| err.to_string())
    })
        .map_err(io::Error::other)?;

    Ok((&app).into())
}

#[command]
#[specta::specta]
pub async fn apply_app_update(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    app_id: String,
    granted_permissions_input: SageGrantedPermissionsInput,
) -> Result<SageAppView> {
    let _base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let resolved = crate::runtime::resolve_stopped_app(&app_handle, &app_id)
        .await
        .map_err(|err| {
            io::Error::other(format!(
                "failed to resolve stopped app {app_id} for update: {err}"
            ))
        })?;

    let pending = resolved.try_with_app(|app| {
        app.try_with(|sage_app| {
            let user_app = sage_app
                .as_user()
                .ok_or_else(|| anyhow::anyhow!("system app cannot receive user update"))?;

            Ok::<_, anyhow::Error>(user_app.pending_update().cloned())
        })
    })?;

    let pending = match pending {
        Some(pending) => pending,
        None => fetch_pending_update_for_resolved_stopped_app(&resolved)
            .await?
            .ok_or_else(|| io::Error::other(format!("app {app_id} has no available update")))?,
    };

    let app_path = resolved.with_app(|app| app.with(SageApp::app_path));

    let snapshot = download_url_snapshot(
        &app_path,
        pending.app_url(),
        pending.manifest(),
        pending.manifest_hash(),
    )
        .await
        .map_err(|err| io::Error::other(format!("failed to download update snapshot: {err}")))?;

    resolved
        .try_with_app(|app| {
            app.try_mutate(|sage_app| {
                let granted_permissions = granted_permissions_input
                    .resolve(pending.manifest().permissions())
                    .map_err(|err| anyhow::anyhow!("invalid update permissions: {err}"))?;

                sage_app.apply_update(&pending, granted_permissions, snapshot)?;

                Ok::<_, anyhow::Error>(())
            })
        })
        .map_err(io::Error::other)?;

    Ok(resolved.with_app(|app| app.into()))
}
