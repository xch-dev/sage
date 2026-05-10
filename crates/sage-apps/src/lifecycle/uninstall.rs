use crate::AppsHostState;
use crate::bridge::methods::system::emit_listed_apps_changed;
use crate::host::{AppState, Result};
use crate::lifecycle::{apps_root, enqueue_retired_app_origin, record_storage_cleanup_failure};
use crate::runtime::{ResolveStoppedError, resolve_stopped_app, run_verified_storage_clear_cycle};
use std::time::Duration;
use std::{fs, io};
use tauri::{AppHandle, Manager, State, command};
use tokio::time::timeout;

#[command]
#[specta::specta]
pub async fn uninstall_app(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
) -> Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let dir = apps_root(&base_path).join(&app_id);

    let resolved_app = match resolve_stopped_app(&app_handle, &app_id).await {
        Ok(app) => Some(app),
        Err(ResolveStoppedError::AppDirMissing) => {
            tracing::error!(
                "[uninstall_app] app dir missing/unresolvable {app_id}; removing dir only"
            );
            None
        }

        Err(ResolveStoppedError::CloseAttemptsHit) => {
            tracing::error!("[uninstall_app] close attempts hit {app_id}");
            return Err(io::Error::other(
                "failed to uninstall app because runtime could not be stopped",
            )
            .into());
        }
    };

    if let Some(resolved_app) = resolved_app {
        let cleanup_result = timeout(
            Duration::from_millis(2_000),
            run_verified_storage_clear_cycle(&app_handle, &resolved_app),
        )
        .await
        .unwrap_or_else(|_| {
            tracing::error!("[uninstall_app] storage cleanup timed out for {app_id}");
            Err("storage cleanup timed out".to_string())
        });
        resolved_app.try_with_app(|installed| {
            if cleanup_result.is_ok() {
                enqueue_retired_app_origin(installed, false)
                    .map_err(|_| io::Error::other("failed to retire app origin"))?;
            } else {
                record_storage_cleanup_failure(&base_path, installed, "storage cleanup failed")
                    .map_err(|_| io::Error::other("failed to record storage cleanup failure"))?;

                enqueue_retired_app_origin(installed, true)
                    .map_err(|_| io::Error::other("failed to retire app origin"))?;
            }

            Ok::<(), crate::host::SageAppsError>(())
        })?;
    }

    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|err| {
            io::Error::other(format!(
                "failed to remove installed app {} at {}: {err}",
                app_id,
                dir.display()
            ))
        })?;
    }

    let host_state: State<'_, AppsHostState> = app_handle.state();
    emit_listed_apps_changed(&app_handle, &host_state, &base_path).await;

    Ok(())
}
