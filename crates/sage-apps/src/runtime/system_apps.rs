use std::collections::BTreeMap;

use tauri::{AppHandle, State};

use crate::runtime::start::{create_runtime, CreateRuntimeArgs};
use crate::runtime::{RuntimeChangeSet, SageAppRuntimeMode, SageAppRuntimeRecord, SageAppRuntimeVisibility, SharedRuntime};
use crate::system_apps::{SYSTEM_APP_APP_INSTALL_ID, SYSTEM_APP_APP_UPDATE_ID, SYSTEM_APP_BRIDGE_APPROVAL_ID, SYSTEM_APP_DONATION_ID};
use crate::types::{AppModalPresentation, AppPresentation};
use crate::AppsHostState;
use crate::bridge::state::pending_approval_app_ids;
use crate::runtime::manager::sync_modal_runtime_visibility;
use crate::runtime::stop::{kill_runtime_inner};

pub(crate) async fn start_system_app_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SharedRuntime, String> {
    let runtime = create_runtime(app_handle, apps_state, args).await?;

    let host_window_label =
        runtime.with_runtime(SageAppRuntimeRecord::host_window_label);

    let mut changes = RuntimeChangeSet::default();
    changes.runtimes_changed();

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &host_window_label,
        &mut changes,
    )
        .await?;

    changes.emit(app_handle, apps_state).await;

    Ok(runtime)
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
            presentation: AppPresentation::Modal(AppModalPresentation::over_launchpad(40)),
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
            presentation: AppPresentation::Modal(AppModalPresentation::over_app_and_launchpad(target_app_id, 50)),
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
    target_app_ids: Vec<String>,
) -> Result<SharedRuntime, String> {
    start_system_app_runtime(
        app,
        apps_state,
        CreateRuntimeArgs {
            app_id: SYSTEM_APP_BRIDGE_APPROVAL_ID.to_string(),
            presentation: AppPresentation::Modal(AppModalPresentation::over_apps(target_app_ids, 100)),
            mode: SageAppRuntimeMode::Inline,
            visibility: SageAppRuntimeVisibility::Visible,
            debug_layout: false,
            query: BTreeMap::new(),
        },
    )
        .await
}

pub(crate) async fn start_donation_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    target_app_id: String,
    query: BTreeMap<String, String>,
) -> Result<SharedRuntime, String> {
    start_system_app_runtime(
        app,
        apps_state,
        CreateRuntimeArgs {
            app_id: SYSTEM_APP_DONATION_ID.to_string(),
            presentation: AppPresentation::Modal(
                AppModalPresentation::over_apps(vec![target_app_id], 45),
            ),
            mode: SageAppRuntimeMode::Inline,
            visibility: SageAppRuntimeVisibility::Visible,
            debug_layout: false,
            query,
        },
    )
        .await
}

pub(crate) async fn sync_bridge_approval_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    let mut changes = RuntimeChangeSet::default();

    let visible_over_app_ids = pending_approval_app_ids(apps_state).await;
    if visible_over_app_ids.is_empty() {
        let _ = kill_runtime_inner(
            app_handle,
            apps_state,
            SYSTEM_APP_BRIDGE_APPROVAL_ID,
            "no_pending_approvals",
            &mut changes,
        )
            .await;

        changes.emit(app_handle, apps_state).await;

        return Ok(());
    }

    let approval_runtime = start_bridge_approval_runtime(
        app_handle,
        apps_state,
        visible_over_app_ids.clone(),
    )
        .await?;

    let presentation_changed = approval_runtime.with_runtime_mut(|runtime| {
        runtime.update_modal_presentation_list(visible_over_app_ids)
    })?;

    let mut changes = RuntimeChangeSet::default();

    if presentation_changed {
        changes.runtimes_changed();
    }

    let host_window_label =
        approval_runtime.with_runtime(SageAppRuntimeRecord::host_window_label);

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &host_window_label,
        &mut changes,
    )
        .await?;

    changes.emit(app_handle, apps_state).await;

    Ok(())
}
