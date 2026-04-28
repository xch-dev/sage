use crate::AppsHostState;
use crate::runtime::emit_runtime_manager_runtimes_changed;
use crate::runtime::state::read::{
    find_runtime_by_runtime_id_optional, find_runtime_id_by_app_id_optional, get_runtime_by_app_id,
};
use crate::runtime::state::remove::{
    remove_before_stop_listeners_by_app_id, remove_pending_stop_ready,
    remove_runtime_by_runtime_id, remove_runtime_id_by_app_id,
};
use crate::runtime::state::types::{SageAppRuntimeRecord, SageLifecycleBeforeStopDetail};
use crate::runtime::state::write::write_pending_stop_ready;
use crate::runtime::webview_locator::find_webview_in_sage_window;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

const BEFORE_STOP_TIMEOUT_MS: u64 = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemKillRuntimeResult {
    pub ok: bool,
    pub app_id: String,
}

pub async fn kill_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    reason: &str,
) -> Result<SystemKillRuntimeResult, String> {
    let _ = get_runtime_by_app_id(apps_state, app_id).await?;

    close_runtime_internal_with_reason(app, apps_state, app_id, reason).await?;

    Ok(SystemKillRuntimeResult {
        ok: true,
        app_id: app_id.to_string(),
    })
}

pub async fn close_runtime_internal(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<(), String> {
    close_runtime_internal_with_reason(app, apps_state, app_id, "host_close").await
}

pub async fn close_runtime_internal_with_reason(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    reason: &str,
) -> Result<(), String> {
    let Some(runtime_id) = find_runtime_id_by_app_id_optional(apps_state, app_id).await else {
        return Ok(());
    };
    let Some(runtime) = find_runtime_by_runtime_id_optional(apps_state, &runtime_id).await else {
        remove_runtime_id_by_app_id(apps_state, app_id).await;
        return Ok(());
    };

    let _ = wait_for_before_stop_ack(app, apps_state, &runtime, reason).await;

    if let Some(webview) = find_webview_in_sage_window(app, runtime.webview_label()) {
        let _ = webview.close();
    }

    remove_runtime_by_runtime_id(apps_state, &runtime_id).await;
    remove_runtime_id_by_app_id(apps_state, app_id).await;
    remove_before_stop_listeners_by_app_id(apps_state, app_id).await;
    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(())
}

async fn wait_for_before_stop_ack(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    runtime: &SageAppRuntimeRecord,
    reason: &str,
) -> Result<(), String> {
    let has_listener = {
        let listeners = apps_state
            .runtime
            .before_stop_listeners_by_app_id
            .lock()
            .await;
        listeners.contains(runtime.app_id())
    };

    if !has_listener {
        return Ok(());
    }
    let Some(app_webview) = find_webview_in_sage_window(app, runtime.webview_label()) else {
        return Ok(());
    };

    let request_id = Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();

    write_pending_stop_ready(apps_state, &request_id, tx).await;

    let detail = SageLifecycleBeforeStopDetail {
        request_id: request_id.clone(),
        reason: Some(reason.to_string()),
        app_id: Some(runtime.app_id().to_string()),
        runtime_id: Some(runtime.runtime_id().to_string()),
    };

    let _ = app_webview.emit("sage-lifecycle:before-stop", detail);
    let _ = timeout(Duration::from_millis(BEFORE_STOP_TIMEOUT_MS), rx).await;

    remove_pending_stop_ready(apps_state, &request_id).await;

    Ok(())
}
