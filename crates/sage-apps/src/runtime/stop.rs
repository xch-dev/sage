use std::fmt::Display;
use crate::AppsHostState;
use crate::runtime::{find_active_taskbar_runtime, find_impostor_runtime_by_victim_app_id_optional, GetRuntimeError, SageAppRuntimeRecord, SharedRuntime};
use crate::runtime::state::{find_runtime_by_runtime_id_optional, find_runtime_id_by_app_id_optional, get_runtime_by_app_id, remove_before_stop_listeners_by_app_id, remove_pending_stop_ready, remove_runtime_by_runtime_id, remove_runtime_id_by_app_id, write_pending_stop_ready, remove_impostor_runtime_by_victim_app_id};
use crate::runtime::webview_locator::find_webview_in_sage_window;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;
use tauri::{AppHandle, State};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;
use crate::bridge::emit_user_runtime_event_to_app_id;
use crate::bridge::methods::user::app::events::BeforeStopEvent;
use crate::runtime::events::{emit_active_taskbar_runtime_changed, emit_runtime_manager_runtimes_changed};

const BEFORE_STOP_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemKillRuntimeResult {
    pub ok: bool,
    pub app_id: String,
}


#[derive(Debug, Copy, Clone)]
pub enum SystemKillRuntimeError {
    NotFound
}

impl Display for SystemKillRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemKillRuntimeError::NotFound => write!(f, "Runtime not found"),
        }
    }
}

pub(crate) async fn kill_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    reason: &str,
) -> Result<(), SystemKillRuntimeError> {
    let shared_runtime = get_runtime_by_app_id(apps_state, app_id).await.map_err(|e| match e {
        GetRuntimeError::NotFound => SystemKillRuntimeError::NotFound
    })?;
    let host_window_label = shared_runtime.with_runtime(SageAppRuntimeRecord::host_window_label);
    let active_taskbar_runtime = find_active_taskbar_runtime(
        apps_state,
        &host_window_label
    ).await;

    close_runtime_internal_with_reason(app_handle, apps_state, app_id, reason).await;

    if let Some(active_taskbar_runtime) = active_taskbar_runtime && active_taskbar_runtime.app_id() == app_id {
        emit_active_taskbar_runtime_changed(app_handle, apps_state, &host_window_label, None).await;
    }

    Ok(())
}

pub(crate) async fn close_runtime_internal(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) {
    close_runtime_internal_with_reason(app, apps_state, app_id, "host_close").await;
}

pub(super) async fn close_runtime_internal_with_reason(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    _reason: &str,
) {
    // Impostor?
    if let Some(impostor) = find_impostor_runtime_by_victim_app_id_optional(apps_state, app_id).await {
        let webview_label = impostor.webview_label();

        if let Some(webview) = find_webview_in_sage_window(app, &webview_label) {
            let _ = webview.close();
        }

        remove_impostor_runtime_by_victim_app_id(apps_state, app_id).await;
        return;
    }

    // Legit?
    let Some(runtime_id) = find_runtime_id_by_app_id_optional(apps_state, app_id).await else {
        return;
    };
    let Some(runtime) = find_runtime_by_runtime_id_optional(apps_state, &runtime_id).await else {
        remove_runtime_id_by_app_id(apps_state, app_id).await;
        return;
    };

    let _ = wait_for_before_stop_ack(app, apps_state, &runtime).await;

    if let Some(webview) = find_webview_in_sage_window(app, &runtime.app().webview_label()) {
        let _ = webview.close();
    }

    remove_runtime_by_runtime_id(apps_state, &runtime_id).await;
    remove_runtime_id_by_app_id(apps_state, app_id).await;
    remove_before_stop_listeners_by_app_id(apps_state, app_id).await;
    emit_runtime_manager_runtimes_changed(app, apps_state).await;
}

async fn wait_for_before_stop_ack(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    runtime: &SharedRuntime,
) -> Result<(), String> {
    let has_listener = {
        let listeners = apps_state
            .runtime
            .before_stop_listeners_by_app_id
            .lock()
            .await;
        listeners.contains(&runtime.app_id())
    };

    if !has_listener {
        return Ok(());
    }

    let request_id = Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();

    write_pending_stop_ready(apps_state, &request_id, tx).await;

    let _ = emit_user_runtime_event_to_app_id(app_handle, &runtime.app_id(), BeforeStopEvent::new(&request_id)).await;
    let _ = timeout(Duration::from_millis(BEFORE_STOP_TIMEOUT_MS), rx).await;

    remove_pending_stop_ready(apps_state, &request_id).await;

    Ok(())
}
