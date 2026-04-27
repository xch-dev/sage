use crate::AppsHostState;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::{RustBridgeResponse, response_channel_for_runtime_kind};
use crate::runtime::app_id_from_webview_label;
use crate::runtime::state::read::find_runtime_by_app_id_optional;
use crate::runtime::state::types::SageAppRuntimeKind;
use crate::runtime::webview_locator::get_webview_in_sage_window;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) fn emit_bridge_response_to_source(
    app: &AppHandle,
    app_webview_label: &str,
    response: &RustBridgeResponse,
) -> Result<(), String> {
    let (runtime_kind, _) = app_id_from_webview_label(app_webview_label)
        .ok_or_else(|| format!("invalid webview label for bridge response: {app_webview_label}"))?;
    get_webview_in_sage_window(app, app_webview_label)?
        .emit(&response_event_for_runtime_kind(runtime_kind), response)
        .map_err(|err| format!("failed to emit bridge response: {err}"))
}

pub(crate) async fn emit_bridge_event_to_app_id(
    app: &AppHandle,
    app_id: &str,
    event: EventForApp,
) -> Result<(), String> {
    let apps_state = app.state::<AppsHostState>();

    let Some(runtime) = find_runtime_by_app_id_optional(&apps_state, app_id).await else {
        return Ok(());
    };

    get_webview_in_sage_window(app, &runtime.webview_label)?
        .emit(&event_event_for_runtime_kind(runtime.runtime_kind), event)
        .map_err(|err| format!("failed to emit bridge event: {err}"))
}

fn response_event_for_runtime_kind(runtime_kind: SageAppRuntimeKind) -> String {
    format!(
        "{}:response",
        response_channel_for_runtime_kind(runtime_kind)
    )
}

fn event_event_for_runtime_kind(runtime_kind: SageAppRuntimeKind) -> String {
    format!("{}:event", response_channel_for_runtime_kind(runtime_kind))
}
