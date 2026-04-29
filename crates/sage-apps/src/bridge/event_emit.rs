use crate::AppsHostState;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::{RustBridgeResponse, response_channel_for_app};
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::runtime::{app_id_from_webview_label, find_runtime_by_app_id_optional, get_runtime_by_app_id};
use tauri::{AppHandle, Emitter, Manager};
use crate::types::{SharedSageApp};

pub(crate) async fn emit_bridge_response_to_source(
    app: &AppHandle,
    app_webview_label: &str,
    response: &RustBridgeResponse,
) -> Result<(), String> {
    let app_id = app_id_from_webview_label(app_webview_label)
        .ok_or_else(|| format!("invalid webview label for bridge response: {app_webview_label}"))?;
    let shared_runtime = get_runtime_by_app_id(&app.state(), app_id).await?;
    let response_event = shared_runtime.with_app(response_event_for_app);

    get_webview_in_sage_window(app, app_webview_label)?
        .emit(&response_event, response)
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

    let (webview_label, event_name) = runtime.with_runtime(|runtime| {
        (
            runtime.webview_label().to_string(),
            event_event_for_app(runtime.app()),
        )
    });

    get_webview_in_sage_window(app, &webview_label)?
        .emit(&event_name, event)
        .map_err(|err| format!("failed to emit bridge event: {err}"))
}

fn response_event_for_app(app: &SharedSageApp) -> String {
    format!(
        "{}:response",
        response_channel_for_app(app)
    )
}

fn event_event_for_app(app: &SharedSageApp) -> String {
    format!("{}:event", response_channel_for_app(app))
}
