use serde::Serialize;
use specta::Type;
use crate::AppsHostState;
use crate::bridge::{RustBridgeResponse};
use crate::runtime::webview_locator::{get_sage_webview, get_webview_in_sage_window};
use crate::runtime::resolve_possibly_impostor_running_app_immediate;
use tauri::{AppHandle, Emitter, Manager};
use crate::types::{SharedSageApp};

const SAGE_RUNTIME_EVENT_NAME: &str = "apps:runtime-event";

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeEvent<T: AppRuntimeEvent> {
    #[serde(rename = "type")]
    pub event_type: &'static str,

    pub payload: T,
}

pub(crate) fn runtime_event<T: AppRuntimeEvent>(payload: T) -> RuntimeEvent<T> {
    RuntimeEvent {
        event_type: T::TYPE,
        payload,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppRuntimeEventRail {
    User,
    System,
}

impl AppRuntimeEventRail {
    pub(crate) fn event_name(self) -> &'static str {
        match self {
            Self::User => "sage-bridge:event",
            Self::System => "sage-system-bridge:event",
        }
    }
}

pub(crate) trait AppRuntimeEvent: Serialize + Type + Clone {
    const TYPE: &'static str;
    const RAIL: AppRuntimeEventRail;
}

pub(crate) async fn emit_runtime_event_to_app_id<T>(
    app_handle: &AppHandle,
    app_id: &str,
    event: T,
) -> Result<(), String>
where
    T: AppRuntimeEvent,
{
    let apps_state = app_handle.state::<AppsHostState>();

    let runtime = resolve_possibly_impostor_running_app_immediate(&apps_state, app_id)?;

    let webview_label = runtime.identity_webview_label();

    get_webview_in_sage_window(app_handle, &webview_label)?
        .emit(T::RAIL.event_name(), runtime_event(event))
        .map_err(|err| format!("failed to emit runtime event: {err}"))
}

pub(crate) fn emit_runtime_event_to_sage_webview<T>(
    app_handle: &AppHandle,
    event: T,
) -> Result<(), String>
where
    T: AppRuntimeEvent,
{
    get_sage_webview(app_handle)?
        .emit(SAGE_RUNTIME_EVENT_NAME, runtime_event(event))
        .map_err(|err| format!("failed to emit runtime event to Sage webview: {err}"))
}

pub(super) async fn emit_bridge_response_to_source(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    response: &RustBridgeResponse,
) -> Result<(), String> {
    get_webview_in_sage_window(app_handle, &app.webview_label())?
        .emit("sage-bridge:response", response)
        .map_err(|err| format!("failed to emit bridge response: {err}"))
}
