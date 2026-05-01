use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppsHostState;
use crate::bridge::methods::system::RuntimeManagerRuntimesChangedEvent;
use crate::runtime::{SageAppRuntimeRecordView, SharedRuntime};
use crate::runtime::state::{
    get_runtime_by_app_id, list_runtimes,
};
use crate::runtime::webview_locator::{find_sage_window, get_webview_in_sage_window};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTargetParams {
    pub app_id: String,
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

    let event = RuntimeManagerRuntimesChangedEvent::new(
        runtime_records,
    );

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
    let runtime = get_runtime_by_app_id(apps_state, app_id).await.map_err(|e| e.to_string())?;

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    let webview = get_webview_in_sage_window(app, &webview_label)?;

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
    let runtime = get_runtime_by_app_id(apps_state, app_id).await.map_err(|e| e.to_string())?;

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    let webview = get_webview_in_sage_window(app, &webview_label)?;

    webview
        .hide()
        .map_err(|err| format!("failed to hide webview: {err}"))?;

    runtime.with_runtime_mut(|runtime| runtime.mark_hidden());

    emit_runtime_manager_runtimes_changed(app, apps_state).await;

    Ok(runtime)
}
