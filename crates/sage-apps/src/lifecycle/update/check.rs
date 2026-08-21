use std::io;

use tauri::{AppHandle, State};

use crate::{
    AppMutationManager, AppsHostState, RecoverableUserSageApp, ResolvedApp, Result, SageApp,
    SageAppSnapshot, SageAppUrlPreview, SharedSageApp, UserSageAppPendingUpdate, UserSageAppSource,
    emit_pending_update_changed, fetch_url_manifest, fetch_url_manifest_preview, resolve_app,
};

pub(crate) enum AppUpdatePreviewResult {
    None,
    AlreadyPending(SageAppUrlPreview),
    New(SageAppUrlPreview),
}

pub(crate) async fn check_app_update_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<SageAppUrlPreview>> {
    let Ok(resolved) = resolve_app(app_handle, app_id).await else {
        return check_recoverable_app_update_inner(apps_state, app_id).await;
    };

    let app = resolved.clone_app_for_operation();
    let mutation_manager = AppMutationManager::new(app_handle, apps_state);

    let preview = match preview_app_update(&resolved).await? {
        AppUpdatePreviewResult::None => {
            mutation_manager
                .mutate_shared_app(&app, |ctx| {
                    Box::pin(async move {
                        ctx.draft_mut().app_mut().set_pending_update(None)?;

                        Ok(())
                    })
                })
                .await
                .map_err(io::Error::other)?;

            emit_pending_update_changed(app_handle, apps_state, &app).await;

            return Ok(None);
        }

        AppUpdatePreviewResult::AlreadyPending(preview) => {
            emit_pending_update_changed(app_handle, apps_state, &app).await;
            return Ok(Some(preview));
        }

        AppUpdatePreviewResult::New(preview) => preview,
    };

    let pending_update = match fetch_pending_update(&app).await {
        Ok(pending_update) => pending_update,

        Err(err) => {
            tracing::warn!(
                error = %err,
                app_id = %app_id,
                "app update exists but pending update could not be prepared"
            );

            return Ok(Some(preview));
        }
    };

    mutation_manager
        .mutate_shared_app(&app, |ctx| {
            Box::pin(async move {
                ctx.draft_mut()
                    .app_mut()
                    .set_pending_update(pending_update)?;

                Ok(())
            })
        })
        .await
        .map_err(io::Error::other)?;

    tracing::info!(app_id = %app_id, "app update prepared successfully");

    emit_pending_update_changed(app_handle, apps_state, &app).await;

    Ok(Some(preview))
}

async fn check_recoverable_app_update_inner(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<SageAppUrlPreview>> {
    let operation_lock = apps_state.operation_lock_for_app(app_id);
    let _operation_guard = operation_lock.lock().await;

    let Some(candidate) = fetch_recoverable_app_update(apps_state, app_id).await? else {
        return Ok(None);
    };

    Ok(Some(candidate.1))
}

pub(crate) async fn fetch_recoverable_app_update(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<(RecoverableUserSageApp, SageAppUrlPreview)>> {
    let recoverable = apps_state
        .db
        .get_recoverable_user_app(app_id)
        .await
        .map_err(|err| {
            io::Error::other(format!(
                "failed to load recovery state for app {app_id}: {err}"
            ))
        })?
        .ok_or_else(|| io::Error::other(format!("app {app_id} is not installed")))?;

    let UserSageAppSource::Url { app_url } = recoverable.source() else {
        return Ok(None);
    };

    let (manifest_preview, manifest_hash) = fetch_url_manifest_preview(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::new(app_url, manifest_preview, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    Ok(Some((recoverable, preview)))
}

async fn preview_app_update(app: &ResolvedApp) -> Result<AppUpdatePreviewResult> {
    let shared_sage_app = app.clone_app_for_operation();

    let deps = shared_sage_app.with(|app| match app {
        SageApp::System(_) => None,
        SageApp::User(user_app) => Some((
            user_app.source().clone(),
            user_app.common().active_snapshot().clone(),
            user_app.pending_update().cloned(),
        )),
    });

    let Some((source, active_snapshot, existing_pending)) = deps else {
        return Ok(AppUpdatePreviewResult::None);
    };

    let app_url = match source {
        UserSageAppSource::Url { app_url } => app_url,
        UserSageAppSource::Zip => return Ok(AppUpdatePreviewResult::None),
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
        return Ok(AppUpdatePreviewResult::None);
    }

    if let Some(existing_pending) = existing_pending
        && let Some(full_manifest) = preview.full_manifest()
        && existing_pending.manifest_hash() == preview.manifest_hash()
        && existing_pending.manifest() == full_manifest
    {
        return Ok(AppUpdatePreviewResult::AlreadyPending(preview));
    }

    Ok(AppUpdatePreviewResult::New(preview))
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
