use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppsHostState;
use crate::bridge::methods::system::RuntimeManagerRuntimesChangedEvent;
use crate::runtime::state::{
    SageAppRuntimeKind, SageAppRuntimeRecord, get_runtime_by_app_id, list_runtimes,
    write_runtime_and_emit_changed,
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

    let system_runtime_webview_labels = {
        let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;

        by_runtime_id
            .values()
            .filter(|record| {
                !record.internal() && record.runtime_kind() == SageAppRuntimeKind::System
            })
            .map(|record| record.webview_label().to_string())
            .collect::<Vec<_>>()
    };

    let event = RuntimeManagerRuntimesChangedEvent::new("sage-system-bridge".to_string(), runtimes);

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
) -> Result<SageAppRuntimeRecord, String> {
    let mut runtime = get_runtime_by_app_id(apps_state, app_id).await?;
    let webview = get_webview_in_sage_window(app, runtime.webview_label())?;

    webview
        .show()
        .map_err(|err| format!("failed to show webview: {err}"))?;

    webview
        .set_focus()
        .map_err(|err| format!("failed to focus webview: {err}"))?;

    runtime.mark_visible();

    write_runtime_and_emit_changed(app, apps_state, runtime.clone()).await;
    Ok(runtime)
}

pub(crate) async fn hide_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SageAppRuntimeRecord, String> {
    let mut runtime = get_runtime_by_app_id(apps_state, app_id).await?;
    let webview = get_webview_in_sage_window(app, runtime.webview_label())?;

    webview
        .hide()
        .map_err(|err| format!("failed to hide webview: {err}"))?;

    runtime.mark_hidden();

    write_runtime_and_emit_changed(app, apps_state, runtime.clone()).await;
    Ok(runtime)
}
