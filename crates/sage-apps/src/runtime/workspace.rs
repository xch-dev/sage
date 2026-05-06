use tauri::{AppHandle, State};

use crate::runtime::state::{activate_apps_workspace, deactivate_apps_workspace, is_apps_workspace_active};
use crate::runtime::hide_all_runtimes;
use crate::AppsHostState;
use crate::runtime::events::emit_runtime_manager_runtimes_changed;
use crate::runtime::manager::sync_modal_runtime_visibility;
use crate::runtime::webview_locator::get_sage_window;

pub(in crate::runtime) async fn ensure_apps_workspace_active(
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    if !is_apps_workspace_active(apps_state).await {
        return Err("Apps workspace is not active".to_string());
    }

    Ok(())
}

pub(in crate::runtime) async fn enter_apps_workspace(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    activate_apps_workspace(apps_state).await;
    let sage_window = get_sage_window(app_handle)?;
    hide_all_runtimes(app_handle, apps_state).await?;

    sync_modal_runtime_visibility(app_handle, apps_state, sage_window.label()).await?;
    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;

    Ok(())
}

pub(in crate::runtime) async fn leave_apps_workspace(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    deactivate_apps_workspace(apps_state).await;
    hide_all_runtimes(app_handle, apps_state).await?;

    Ok(())
}
