use std::collections::BTreeMap;
use std::io;

use tauri::{command, AppHandle, State};

use crate::bridge::methods::system::{emit_listed_apps_changed, emit_pending_update_changed};
use crate::host::AppState;
use crate::host::Result;
use crate::lifecycle::download_url_snapshot;
use crate::lifecycle::update::logic::{check_app_update_for_app, fetch_pending_update};
use crate::runtime::{resolve_app, start_app_update_runtime};
use crate::types::{
    SageApp, SageAppUrlPreview, SageAppView, SageGrantedPermissionsInput,
    UserSageAppPendingUpdateView,
};
use crate::AppsHostState;

#[command]
#[specta::specta]
pub async fn check_app_update(
    state: State<'_, AppState>,
    apps_state: State<'_, AppsHostState>,
    app_handle: AppHandle,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let resolved = resolve_app(&app_handle, &app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let preview = check_app_update_for_app(&resolved).await?;

    let Some(preview) = preview else {
        return Ok(None);
    };

    let app = resolved.clone_app_for_operation();

    match fetch_pending_update(&app).await {
        Ok(Some(pending)) => {
            app.try_mutate(|sage_app| {
                sage_app
                    .set_pending_update(Some(pending))
                    .map_err(|err| err.to_string())
            })
                .map_err(io::Error::other)?;

            emit_pending_update_changed(&app_handle, &apps_state, &app).await;
            emit_listed_apps_changed(&app_handle, &apps_state, &base_path).await;

            if should_review_pending_update(&app) {
                open_update_runtime(
                    &app_handle,
                    &apps_state,
                    &app_id,
                    None,
                    "failed to start app update review runtime",
                )
                    .await;
            }
        }

        Ok(None) => {
            app.try_mutate(|sage_app| {
                sage_app
                    .set_pending_update(None)
                    .map_err(|err| err.to_string())
            })
                .map_err(io::Error::other)?;

            emit_pending_update_changed(&app_handle, &apps_state, &app).await;
            emit_listed_apps_changed(&app_handle, &apps_state, &base_path).await;
        }

        Err(err) => {
            tracing::warn!(
                error = %err,
                app_id = %app_id,
                "app update exists but pending update could not be prepared"
            );

            open_update_runtime(
                &app_handle,
                &apps_state,
                &app_id,
                Some(err.to_string()),
                "failed to start app update issue runtime",
            )
                .await;
        }
    }

    Ok(Some(preview))
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

    let pending =
        pending.ok_or_else(|| io::Error::other(format!("app {app_id} has no pending update")))?;

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

fn should_review_pending_update(app: &crate::types::SharedSageApp) -> bool {
    app.with(|sage_app| {
        let Some(user_app) = sage_app.as_user() else {
            return false;
        };

        user_app
            .pending_update()
            .map(|pending| {
                UserSageAppPendingUpdateView::from_pending_update(
                    pending,
                    user_app.common().granted_permissions(),
                )
                    .decision()
                    .is_review()
            })
            .unwrap_or(false)
    })
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
