use crate::AppsHostState;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::{RustBridgeResponse};
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::runtime::{find_runtime_by_app_id_optional};
use tauri::{AppHandle, Emitter, Manager};
use crate::types::{SharedSageApp};

pub(crate) async fn emit_bridge_response_to_source(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    response: &RustBridgeResponse,
) -> Result<(), String> {
    get_webview_in_sage_window(app_handle, &app.webview_label())?
        .emit("sage-bridge:response", response)
        .map_err(|err| format!("failed to emit bridge response: {err}"))
}

pub(crate) async fn emit_bridge_event_to_app_id(
    app_handle: &AppHandle,
    app_id: &str,
    event: EventForApp,
) -> Result<(), String> {
    let apps_state = app_handle.state::<AppsHostState>();

    let Some(runtime) = find_runtime_by_app_id_optional(&apps_state, app_id).await else {
        return Ok(());
    };

    let webview_label = runtime.with_runtime(|runtime| {
        runtime.webview_label().to_string()
    });

    get_webview_in_sage_window(app_handle, &webview_label)?
        .emit("sage-bridge:event", event)
        .map_err(|err| format!("failed to emit bridge event: {err}"))
}
