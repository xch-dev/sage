use tauri::{AppHandle, State};

use crate::{AppsHostState, BridgeApprovalsChangedEvent, comms_debug, emit_bridge_response_to_app, emit_system_runtime_event_to_listeners, list_pending_approvals, list_runtimes, PendingBridgeApproval, resolve_running_app, RuntimeManagerActiveTaskbarRuntimeChangedEvent, RuntimeManagerRuntimesChangedEvent, RustBridgeResponse, SageAppRuntimeRecordView, SharedRuntime, SharedSageApp};

pub(crate) async fn emit_bridge_approvals_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let approvals = list_pending_approvals(apps_state).await;

    comms_debug!("bridge_approvals:changed count={}", approvals.len(),);

    let event = BridgeApprovalsChangedEvent::new_from_list(approvals);
    emit_system_runtime_event_to_listeners(app_handle, apps_state, event).await;
}

pub(crate) async fn emit_timeout_for_pending_approval(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    pending: &PendingBridgeApproval,
) -> Result<(), String> {
    let running_app = resolve_running_app(apps_state, &pending.app_id)
        .await
        .map_err(|err| format!("Failed to resolve app: {err}"))?;

    let app = running_app.with_app(SharedSageApp::clone);

    let response = RustBridgeResponse::error(
        &pending.request.id,
        "approval_timeout",
        "Approval request timed out",
    );

    emit_bridge_response_to_app(app_handle, &app, &response).await
}

pub(crate) async fn emit_runtime_manager_runtimes_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    let runtime_records = runtimes
        .iter()
        .map(Into::into)
        .collect::<Vec<SageAppRuntimeRecordView>>();

    let event = RuntimeManagerRuntimesChangedEvent::new(runtime_records);

    emit_system_runtime_event_to_listeners(app_handle, apps_state, event).await;
}

pub(crate) async fn emit_active_taskbar_runtime_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    runtime: Option<&SharedRuntime>,
) {
    let (runtime_id, app_id) = match runtime {
        Some(shared_runtime) => shared_runtime
            .with_runtime(|record| (Some(record.runtime_id()), Some(record.app_id().clone()))),
        None => (None, None),
    };
    let () = emit_system_runtime_event_to_listeners(
        app_handle,
        apps_state,
        RuntimeManagerActiveTaskbarRuntimeChangedEvent {
            host_window_label: host_window_label.to_string(),
            app_id,
            runtime_id,
        },
    )
    .await;
}
