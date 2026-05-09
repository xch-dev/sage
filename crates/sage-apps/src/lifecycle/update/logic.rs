use std::collections::BTreeMap;
use std::io;
use tauri::{AppHandle, State};
use crate::AppsHostState;
use crate::bridge::methods::system::{emit_pending_update_changed};
use crate::host::Result;
use crate::lifecycle::{download_url_snapshot, fetch_url_manifest, fetch_url_manifest_preview};
use crate::runtime::{resolve_app, start_app_update_runtime};
use crate::types::{ResolvedApp, SageApp, SageAppSnapshot, SageAppUrlPreview, SageAppView, SageGrantedPermissionsInput, SharedSageApp, UserSageAppPendingUpdate, UserSageAppSource};

pub(super) async fn check_app_update_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<SageAppUrlPreview>> {
    let resolved = resolve_app(app_handle, app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let app = resolved.clone_app_for_operation();

    let Some(preview) = preview_app_update(&resolved).await? else {
        app.try_mutate(|sage_app| {
            sage_app
                .set_pending_update(None)
                .map_err(|err| err.to_string())
        })
            .map_err(io::Error::other)?;

        emit_pending_update_changed(app_handle, apps_state, &app).await;

        return Ok(None);
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

    app.try_mutate(|sage_app| {
        sage_app
            .set_pending_update(pending_update)
            .map_err(|err| err.to_string())
    })
        .map_err(io::Error::other)?;

    println!("App update prepared successfully for app: {}", app_id);
    emit_pending_update_changed(app_handle, apps_state, &app).await;

    Ok(Some(preview))
}

pub(crate) async fn apply_app_update_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    granted_permissions_input: Option<SageGrantedPermissionsInput>,
) -> Result<SageAppView> {
    let resolved = crate::runtime::resolve_stopped_app(app_handle, app_id)
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

    let pending =
        pending.ok_or_else(|| io::Error::other(format!("app {app_id} has no pending update")))?;

    let should_review = resolved.with_app(SharedSageApp::should_review_pending_update);

    if should_review && granted_permissions_input.is_none() {
        open_update_runtime(
            app_handle,
            apps_state,
            app_id,
            None,
            "failed to start app update review runtime",
        )
            .await;

        return Ok(resolved.with_app(|app| app.into()));
    }

    let granted_permissions_input = match granted_permissions_input {
        Some(input) => input,
        None => resolved.try_with_app(|app| {
            app.try_with(|sage_app| {
                Ok::<_, anyhow::Error>(
                    SageGrantedPermissionsInput::from(sage_app.common().granted_permissions()),
                )
            })
        })?,
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

    let app = resolved.into_app();

    emit_pending_update_changed(app_handle, apps_state, &app).await;

    Ok(app.into())
}

pub(crate) async fn preview_app_update(
    app: &ResolvedApp,
) -> Result<Option<SageAppUrlPreview>> {
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

pub(crate) async fn fetch_pending_update(
    app: &SharedSageApp,
) -> Result<Option<UserSageAppPendingUpdate>> {
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

async fn open_update_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    issue: Option<String>,
    error_message: &'static str,
) {
    let mut query = BTreeMap::new();
    query.insert("appId".to_string(), app_id.to_string());

    if let Some(issue) = issue {
        query.insert("issue".to_string(), issue);
    }

    let _ = start_app_update_runtime(app_handle, apps_state, app_id.to_string(), query)
        .await
        .map_err(|err| {
            tracing::error!(
                error = %err,
                app_id = %app_id,
                "{error_message}"
            );
            err
        });
}
