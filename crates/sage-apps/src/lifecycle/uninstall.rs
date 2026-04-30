use crate::host::{AppState, Result};
use crate::lifecycle::{
    apps_clear_runtime_browsing_data, apps_root, enqueue_retired_app_origin, record_storage_cleanup_failure,
};
use std::{fs, io};
use tauri::{AppHandle, State, command};
use crate::runtime::{resolve_stopped_app, ResolveStoppedError};

#[command]
#[specta::specta]
pub async fn uninstall_app(
    app: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
) -> Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let resolved_app = match resolve_stopped_app(&app, &app_id).await {
        Ok(app) => app,
        Err(ResolveStoppedError::AppDirMissing) => return Ok(()),
        Err(ResolveStoppedError::CloseAttemptsHit) => {
            return Err(io::Error::other(
                "failed to uninstall app because runtime could not be stopped",
            )
                .into());
        }
    };

    let cleanup_result = apps_clear_runtime_browsing_data(app.clone(), app_id.clone()).await;

    resolved_app.try_with_app(|installed| {
        if let Ok(()) = cleanup_result {
            enqueue_retired_app_origin(installed, false).map_err(|_| {
                io::Error::other("failed to retire app origin")
            })?;
        } else {
            record_storage_cleanup_failure(&base_path, installed, "storage cleanup failed")
                .map_err(|_| {
                    io::Error::other("failed to record storage cleanup failure")
                })?;

            enqueue_retired_app_origin(installed, true).map_err(|_| {
                io::Error::other("failed to retire app origin")
            })?;
        }

        Ok::<(), crate::host::SageAppsError>(())
    })?;

    let dir = apps_root(&base_path).join(&app_id);

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
