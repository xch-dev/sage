use std::collections::BTreeMap;

use tauri::{AppHandle, State};

use crate::runtime::start::{create_runtime, CreateRuntimeArgs};
use crate::runtime::{emit_runtime_manager_runtimes_changed, SageAppRuntimeMode, SageAppRuntimeVisibility, SharedRuntime};
use crate::system_apps::{
    SYSTEM_APP_APP_INSTALL_ID,
    SYSTEM_APP_APP_UPDATE_ID,
    SYSTEM_APP_BRIDGE_APPROVAL_ID,
};
use crate::types::{AppModalPresentation, AppPresentation};
use crate::AppsHostState;

pub(crate) async fn start_system_app_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SharedRuntime, String> {
    let new_runtime = create_runtime(app_handle, apps_state, args).await?;

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;

    Ok(new_runtime)
}

pub(crate) async fn start_app_install_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    query: BTreeMap<String, String>,
) -> Result<SharedRuntime, String> {
    start_system_app_runtime(
        app,
        apps_state,
        CreateRuntimeArgs {
            app_id: SYSTEM_APP_APP_INSTALL_ID.to_string(),
            presentation: AppPresentation::Modal(AppModalPresentation {
                visible_over_app_ids: Vec::new(),
                visible_over_launchpad: true,
            }),
            mode: SageAppRuntimeMode::Inline,
            visibility: SageAppRuntimeVisibility::Visible,
            debug_layout: false,
            query,
        },
    )
        .await
}

pub(crate) async fn start_app_update_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    target_app_id: String,
    query: BTreeMap<String, String>,
) -> Result<SharedRuntime, String> {
    start_system_app_runtime(
        app,
        apps_state,
        CreateRuntimeArgs {
            app_id: SYSTEM_APP_APP_UPDATE_ID.to_string(),
            presentation: AppPresentation::Modal(AppModalPresentation {
                visible_over_app_ids: vec![target_app_id],
                visible_over_launchpad: true,
            }),
            mode: SageAppRuntimeMode::Inline,
            visibility: SageAppRuntimeVisibility::Visible,
            debug_layout: false,
            query,
        },
    )
        .await
}

pub(crate) async fn start_bridge_approval_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    target_app_id: String,
) -> Result<SharedRuntime, String> {
    start_system_app_runtime(
        app,
        apps_state,
        CreateRuntimeArgs {
            app_id: SYSTEM_APP_BRIDGE_APPROVAL_ID.to_string(),
            presentation: AppPresentation::Modal(AppModalPresentation {
                visible_over_app_ids: vec![target_app_id],
                visible_over_launchpad: false,
            }),
            mode: SageAppRuntimeMode::Inline,
            visibility: SageAppRuntimeVisibility::Visible,
            debug_layout: false,
            query: BTreeMap::new(),
        },
    )
        .await
}
