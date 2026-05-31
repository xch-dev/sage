use std::{fs, io};

use tauri::{AppHandle, Manager, State, command};

use crate::AppsHostState;
use crate::bridge::emit_listed_apps_changed;
use crate::host::{AppState, Result};
use crate::lifecycle::apps_root;
use crate::runtime::{ResolveStoppedError, resolve_stopped_app};

#[command]
#[specta::specta]
pub async fn apps_uninstall_app(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
) -> Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let dir = apps_root(&base_path).join(&app_id);

    match resolve_stopped_app(&app_handle, &app_id).await {
        Ok(_) | Err(ResolveStoppedError::AppDirMissing) => {}

        Err(ResolveStoppedError::CloseAttemptsHit) => {
            tracing::error!("[uninstall_app] close attempts hit {app_id}");
            return Err(io::Error::other(
                "failed to uninstall app because runtime could not be stopped",
            )
            .into());
        }
    }

    let host_state: State<'_, AppsHostState> = app_handle.state();

    let mut tx = host_state.db.begin_immediate().await.map_err(|err| {
        io::Error::other(format!(
            "failed to begin uninstall transaction for {app_id}: {err}"
        ))
    })?;

    if let Err(err) = tx.delete_user_app(&app_id).await {
        tx.rollback().await;

        return Err(
            io::Error::other(format!("failed to delete app {app_id} from db: {err}")).into(),
        );
    }

    if let Err(err) = tx.commit().await {
        return Err(io::Error::other(format!(
            "failed to commit uninstall transaction for {app_id}: {err}"
        ))
        .into());
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

    emit_listed_apps_changed(&app_handle, &host_state).await;

    Ok(())
}
