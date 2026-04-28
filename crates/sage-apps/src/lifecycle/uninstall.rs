use crate::host::{AppState, Result};
use crate::lifecycle::{
    apps_clear_runtime_browsing_data, apps_root, enqueue_retired_app_origin,
    read_installed_app_by_id, record_storage_cleanup_failure,
};
use std::{fs, io};
use tauri::{AppHandle, State, command};

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

    let installed = read_installed_app_by_id(&base_path, &app_id).ok();

    if let Some(installed) = &installed {
        let cleanup_result = apps_clear_runtime_browsing_data(app.clone(), app_id.clone()).await;

        match cleanup_result {
            Ok(()) => {
                enqueue_retired_app_origin(&base_path, installed, false).map_err(|err| {
                    io::Error::other(format!(
                        "failed to retire app origin after uninstall cleanup: {err}"
                    ))
                })?;
            }
            Err(err) => {
                record_storage_cleanup_failure(&base_path, installed, &err).map_err(
                    |queue_err| {
                        io::Error::other(format!(
                            "failed to enqueue pending storage cleanup after clear failure ({err}): {queue_err}"
                        ))
                    },
                )?;

                enqueue_retired_app_origin(&base_path, installed, true).map_err(|origin_err| {
                    io::Error::other(format!(
                        "failed to retire app origin after cleanup failure ({err}): {origin_err}"
                    ))
                })?;
            }
        }
    }

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
