use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppsHostState;
use crate::bridge::methods::system::RuntimeManagerRuntimesChangedEvent;
use crate::runtime::state::{
    find_runtime_by_runtime_id_optional, get_runtime_by_app_id, list_runtimes,
};
use crate::runtime::webview_locator::{find_sage_window, get_webview_in_sage_window};
use crate::runtime::{SageAppRuntimeRecordView, SharedRuntime};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTargetParams {
    pub app_id: String,
}

struct RuntimeWindowIdentity {
    runtime_id: String,
    host_window_label: String,
    webview_label: String,
}

fn runtime_window_identity(runtime: &SharedRuntime) -> RuntimeWindowIdentity {
    runtime.with_runtime(|record| RuntimeWindowIdentity {
        runtime_id: record.runtime_id(),
        host_window_label: record.host_window_label().to_string(),
        webview_label: record.webview_label().to_string(),
    })
}

async fn clear_active_runtime_if_matches(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    runtime_id: &str,
) {
    let mut active = apps_state
        .runtime
        .active_runtime_id_by_host_window_label
        .lock()
        .await;

    if active
        .get(host_window_label)
        .is_some_and(|active_runtime_id| active_runtime_id == runtime_id)
    {
        active.remove(host_window_label);
    }
}

async fn set_active_runtime_for_host_window(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    runtime_id: &str,
) -> Option<String> {
    let mut active = apps_state
        .runtime
        .active_runtime_id_by_host_window_label
        .lock()
        .await;

    active.insert(host_window_label.to_string(), runtime_id.to_string())
}

async fn hide_runtime_by_runtime_id_if_present(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) {
    let Some(runtime) = find_runtime_by_runtime_id_optional(apps_state, runtime_id).await else {
        return;
    };

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    if let Ok(webview) = get_webview_in_sage_window(app, &webview_label) {
        let _ = webview.hide();
    }

    runtime.with_runtime_mut(|runtime| runtime.mark_hidden());
}

pub(crate) async fn emit_runtime_manager_runtimes_changed(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    let system_runtime_webview_labels = runtimes
        .iter()
        .filter_map(|runtime| {
            runtime.with_runtime(|record| {
                if record.internal() {
                    return None;
                }

                if !record.app().is_system_app() {
                    return None;
                }

                Some(record.webview_label().to_string())
            })
        })
        .collect::<Vec<_>>();

    let runtime_records = runtimes
        .iter()
        .map(Into::into)
        .collect::<Vec<SageAppRuntimeRecordView>>();

    let event = RuntimeManagerRuntimesChangedEvent::new(runtime_records);

    let Some(sage_window) = find_sage_window(app) else {
        return;
    };

    for system_webview_label in system_runtime_webview_labels {
        if let Some(webview) = sage_window.get_webview(&system_webview_label) {
            let _ = webview.emit("sage-system-bridge:event", event.clone());
        }
    }
}

pub(crate) async fn focus_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    let runtime = get_runtime_by_app_id(apps_state, app_id)
        .await
        .map_err(|e| e.to_string())?;

    let runtime_window_identity = runtime_window_identity(&runtime);

    let previous_runtime_id = set_active_runtime_for_host_window(
        apps_state,
        &runtime_window_identity.host_window_label,
        &runtime_window_identity.runtime_id
    ).await;

    if let Some(previous_runtime_id) = previous_runtime_id
        && previous_runtime_id != runtime_window_identity.runtime_id
    {
        hide_runtime_by_runtime_id_if_present(app, apps_state, &previous_runtime_id).await;
    }

    let webview = get_webview_in_sage_window(app, &runtime_window_identity.webview_label)?;

    webview
        .show()
        .map_err(|err| format!("failed to show webview: {err}"))?;

    webview
        .set_focus()
        .map_err(|err| format!("failed to focus webview: {err}"))?;

    runtime.with_runtime_mut(|runtime| runtime.mark_visible());

    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(runtime)
}

pub(crate) async fn hide_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    let runtime = get_runtime_by_app_id(apps_state, app_id)
        .await
        .map_err(|e| e.to_string())?;

    let runtime_window_identity = runtime_window_identity(&runtime);

    let webview = get_webview_in_sage_window(app, &runtime_window_identity.webview_label)?;

    webview
        .hide()
        .map_err(|err| format!("failed to hide webview: {err}"))?;

    runtime.with_runtime_mut(|runtime| runtime.mark_hidden());

    clear_active_runtime_if_matches(
        apps_state,
        &runtime_window_identity.host_window_label,
        &runtime_window_identity.runtime_id
    ).await;

    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(runtime)
}
