use crate::host::{AppState, Result};
use crate::lifecycle::{
    apps_root, enqueue_retired_app_origin,
    record_storage_cleanup_failure,
};
use crate::runtime::{resolve_stopped_app, run_verified_storage_clear_cycle, ResolveStoppedError};
use std::{fs, io};
use std::time::Duration;
use tauri::{command, AppHandle, State};
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
            eprintln!("[uninstall_app] app dir missing/unresolvable {app_id}; removing dir only");
            None
        }

        Err(ResolveStoppedError::CloseAttemptsHit) => {
            eprintln!("[uninstall_app] close attempts hit {app_id}");
            return Err(io::Error::other(
                "failed to uninstall app because runtime could not be stopped",
            )
                .into());
        }
    };

    if let Some(resolved_app) = resolved_app {
        let cleanup_result = timeout(
            Duration::from_millis(2_000),
            run_verified_storage_clear_cycle(&app_handle, &resolved_app)
        )
            .await
            .unwrap_or_else(|_| {
                eprintln!("[uninstall_app] storage cleanup timed out for {app_id}");
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

    Ok(())
}
