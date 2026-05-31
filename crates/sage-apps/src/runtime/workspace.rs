use tauri::{AppHandle, State};

use crate::{activate_apps_workspace, AppsHostState, deactivate_apps_workspace, get_sage_window, hide_all_runtimes, hide_all_runtimes_inner, is_apps_workspace_active, RuntimeChangeSet, sync_modal_runtime_visibility};

pub(crate) async fn ensure_apps_workspace_active(
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    if !is_apps_workspace_active(apps_state).await {
        return Err("Apps workspace is not active".to_string());
    }

    Ok(())
}

pub(crate) async fn enter_apps_workspace(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    activate_apps_workspace(apps_state).await;

    let sage_window = get_sage_window(app_handle)?;
    let mut changes = RuntimeChangeSet::default();

    hide_all_runtimes_inner(app_handle, apps_state, &mut changes).await?;

    sync_modal_runtime_visibility(app_handle, apps_state, sage_window.label(), &mut changes)
        .await?;

    changes.emit(app_handle, apps_state).await;

    Ok(())
}

pub(crate) async fn leave_apps_workspace(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    deactivate_apps_workspace(apps_state).await;
    hide_all_runtimes(app_handle, apps_state).await?;

    Ok(())
}
