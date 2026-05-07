use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppsHostState;
use crate::bridge::{comms_debug, RustBridgeResponse};
use crate::capabilities::list::{SystemBridgeCapability, UserBridgeCapability};
use crate::lifecycle::{ensure_app_is_enabled_for_scope};
use crate::runtime::webview_locator::{get_sage_webview, get_webview_in_sage_window};
use crate::runtime::{list_runtimes, resolve_possibly_impostor_running_app_immediate};
use crate::types::SharedSageApp;

const SAGE_RUNTIME_EVENT_NAME: &str = "apps:runtime-event";

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeEvent<T> {
    #[serde(rename = "type")]
    pub event_type: &'static str,

    pub payload: T,
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

pub(crate) trait UserRuntimeEvent: Serialize + Type + Clone {
    const TYPE: &'static str;
    const REQUIRED_CAPABILITY: UserBridgeCapability;
}

pub(crate) trait SystemRuntimeEvent: Serialize + Type + Clone {
    const TYPE: &'static str;
    const REQUIRED_CAPABILITY: SystemBridgeCapability;
}

fn runtime_event<T>(event_type: &'static str, payload: T) -> RuntimeEvent<T> {
    RuntimeEvent {
        event_type,
        payload,
    }
}

pub(crate) async fn emit_user_runtime_event_to_listeners<T>(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    event: T,
)
where
    T: UserRuntimeEvent,
{
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    for runtime in runtimes {
        let Some((app_id, can_receive)) = runtime.with_runtime(|record| {
            if record.internal() {
                return None;
            }

            let app = record.app();

            if !app.is_user_app() {
                return None;
            }

            let can_receive = app.is_capability_granted(T::REQUIRED_CAPABILITY.into());

            Some((app.id(), can_receive))
        }) else {
            continue;
        };

        if can_receive {
            let _ = emit_user_runtime_event_to_app_id(app_handle, &app_id, event.clone()).await;
        }
    }

    let _ = emit_user_runtime_event_to_sage_webview(app_handle, event);
}

pub(crate) async fn emit_system_runtime_event_to_listeners<T>(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    event: T,
)
where
    T: SystemRuntimeEvent,
{
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };
    comms_debug!(
        "system:event:listeners type={} required_capability={}",
        T::TYPE,
        T::REQUIRED_CAPABILITY.key(),
    );

    for runtime in runtimes {
        let Some((app_id, can_receive)) = runtime.with_runtime(|record| {
            if record.internal() {
                comms_debug!(
                    "system:event:skip type={} runtime={} reason=internal",
                    T::TYPE,
                    record.app_id(),
                );
                return None;
            }

            let app = record.app();

            if !app.is_system_app() {
                comms_debug!(
                    "system:event:skip type={} app={} reason=not_system",
                    T::TYPE,
                    app.id(),
                );
                return None;
            }

            let can_receive = app.is_capability_granted(T::REQUIRED_CAPABILITY.into());

            comms_debug!(
                "system:event:candidate type={} app={} required_capability={} can_receive={}",
                T::TYPE,
                app.id(),
                T::REQUIRED_CAPABILITY.key(),
                can_receive,
            );

            Some((app.id(), can_receive))
        }) else {
            continue;
        };

        if can_receive {
            comms_debug!(
                "system:event:emit type={} app={}",
                T::TYPE,
                app_id,
            );

            let result = emit_system_runtime_event_to_app_id(
                app_handle,
                &app_id,
                event.clone(),
            ).await;

            if let Err(err) = result {
                comms_debug!(
                    "system:event:emit_failed type={} app={} error={}",
                    T::TYPE,
                    app_id,
                    err,
                );
            }
        }
    }

    let _ = emit_system_runtime_event_to_sage_webview(app_handle, event);
}

pub(crate) async fn emit_user_runtime_event_to_app_id<T>(
    app_handle: &AppHandle,
    app_id: &str,
    event: T,
) -> Result<(), String>
where
    T: UserRuntimeEvent,
{
    emit_runtime_event_to_app_id(
        app_handle,
        app_id,
        AppRuntimeEventRail::User,
        T::TYPE,
        event,
    ).await
}

pub(crate) async fn emit_system_runtime_event_to_app_id<T>(
    app_handle: &AppHandle,
    app_id: &str,
    event: T,
) -> Result<(), String>
where
    T: SystemRuntimeEvent,
{
    emit_runtime_event_to_app_id(
        app_handle,
        app_id,
        AppRuntimeEventRail::System,
        T::TYPE,
        event,
    ).await
}

pub(crate) fn emit_user_runtime_event_to_sage_webview<T>(
    app_handle: &AppHandle,
    event: T,
) -> Result<(), String>
where
    T: UserRuntimeEvent,
{
    get_sage_webview(app_handle)?
        .emit(
            SAGE_RUNTIME_EVENT_NAME,
            runtime_event(T::TYPE, event),
        )
        .map_err(|err| format!("failed to emit runtime event to Sage webview: {err}"))
}

pub(crate) fn emit_system_runtime_event_to_sage_webview<T>(
    app_handle: &AppHandle,
    event: T,
) -> Result<(), String>
where
    T: SystemRuntimeEvent,
{
    get_sage_webview(app_handle)?
        .emit(
            SAGE_RUNTIME_EVENT_NAME,
            runtime_event(T::TYPE, event),
        )
        .map_err(|err| format!("failed to emit runtime event to Sage webview: {err}"))
}

pub(crate) async fn emit_bridge_response_to_app(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    response: &RustBridgeResponse,
) -> Result<(), String> {
    get_webview_in_sage_window(app_handle, &app.webview_label())?
        .emit("sage-bridge:response", response)
        .map_err(|err| format!("failed to emit bridge response: {err}"))
}

async fn emit_runtime_event_to_app_id<T>(
    app_handle: &AppHandle,
    app_id: &str,
    rail: AppRuntimeEventRail,
    event_type: &'static str,
    event: T,
) -> Result<(), String>
where
    T: Serialize + Type + Clone,
{
    let apps_state = app_handle.state::<AppsHostState>();

    let runtime = resolve_possibly_impostor_running_app_immediate(&apps_state, app_id)?;

    ensure_app_is_enabled_for_scope(&app_handle.state(), &runtime.identity_app()).await?;

    let webview_label = runtime.identity_webview_label();
    comms_debug!(
        "runtime:event:emit_to_app app_id={} webview_label={} rail={:?} type={}",
        app_id,
        webview_label,
        rail,
        event_type,
    );

    get_webview_in_sage_window(app_handle, &webview_label)?
        .emit(rail.event_name(), runtime_event(event_type, event))
        .map_err(|err| format!("failed to emit runtime event: {err}"))
}
