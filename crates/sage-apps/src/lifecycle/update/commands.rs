use std::io;

use tauri::{AppHandle, State, command};

use crate::host::AppState;
use crate::host::Result;
use crate::lifecycle::{download_url_snapshot, fetch_url_manifest, fetch_url_manifest_preview};
use crate::runtime::resolve_app;
use crate::types::{
    ResolvedStoppedApp, SageApp, SageAppSnapshot, SageAppUrlPreview, SageAppView,
    SageGrantedPermissionsInput, SharedSageApp, UserSageAppPendingUpdate, UserSageAppSource,
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

    let app = app.clone_app_for_operation();

    let deps = app.with(|app| match app {
        SageApp::System(_) => None,
        SageApp::User(user_app) => Some((
            user_app.source().clone(),
            user_app.common().active_snapshot().clone(),
            user_app.pending_update().cloned(),
        )),
    });

    let Some((source, active_snapshot, existing_pending)) = deps else {
        return Ok(None);
    };

    let app_url = match source {
        UserSageAppSource::Url { app_url } => app_url,
        UserSageAppSource::Zip => return Ok(None),
    };

    let (manifest_preview, manifest_hash) = fetch_url_manifest_preview(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::new(&app_url, manifest_preview, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    if preview.manifest_hash() == active_snapshot.manifest_hash()
        && let Some(full_manifest) = preview.full_manifest()
        && full_manifest == active_snapshot.manifest()
    {
        return Ok(None);
    }

    if let Some(existing_pending) = existing_pending
        && let Some(full_manifest) = preview.full_manifest()
        && existing_pending.manifest_hash() == preview.manifest_hash()
        && existing_pending.manifest() == full_manifest
    {
        return Ok(None);
    }

    Ok(Some(preview))
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

async fn fetch_pending_update(app: &SharedSageApp) -> Result<Option<UserSageAppPendingUpdate>> {
    struct FetchDeps {
        source: UserSageAppSource,
        active_snapshot: SageAppSnapshot,
    }

    let deps = app.with(|app| match app {
        SageApp::System(_) => None,
        SageApp::User(user_app) => Some(FetchDeps {
            source: user_app.source().clone(),
            active_snapshot: user_app.common().active_snapshot().clone(),
        }),
    });
    let Some(deps) = deps else {
        return Ok(None);
    };

    let app_url = match deps.source {
        UserSageAppSource::Url { app_url } => app_url.clone(),
        UserSageAppSource::Zip => return Ok(None),
    };

    let (manifest, manifest_hash) = fetch_url_manifest(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::from_full_manifest(&app_url, manifest, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    let manifest = preview
        .require_full_manifest()
        .map_err(|err| io::Error::other(format!("update manifest is not installable: {err}")))?;

    if preview.manifest_hash() == deps.active_snapshot.manifest_hash()
        && manifest == deps.active_snapshot.manifest()
    {
        return Ok(None);
    }

    Ok(Some(UserSageAppPendingUpdate::new(
        app_url,
        preview.manifest_hash().to_string(),
        manifest.clone(),
    )))
}

async fn fetch_pending_update_for_resolved_stopped_app(
    resolved: &ResolvedStoppedApp,
) -> Result<Option<UserSageAppPendingUpdate>> {
    let deps = resolved.try_with_app(|app| {
        app.try_with(|sage_app| {
            let Some(user_app) = sage_app.as_user() else {
                return Ok(None);
            };

            Ok::<_, anyhow::Error>(Some((
                user_app.source().clone(),
                user_app.common().active_snapshot().clone(),
            )))
        })
    })?;

    let Some((source, active_snapshot)) = deps else {
        return Ok(None);
    };

    let app_url = match source {
        UserSageAppSource::Url { app_url } => app_url,
        UserSageAppSource::Zip => return Ok(None),
    };

    let (manifest, manifest_hash) = fetch_url_manifest(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::from_full_manifest(&app_url, manifest, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    let manifest = preview
        .require_full_manifest()
        .map_err(|err| io::Error::other(format!("update manifest is not installable: {err}")))?;

    if preview.manifest_hash() == active_snapshot.manifest_hash()
        && manifest == active_snapshot.manifest()
    {
        return Ok(None);
    }

    Ok(Some(UserSageAppPendingUpdate::new(
        app_url,
        preview.manifest_hash().to_string(),
        manifest.clone(),
    )))
}
