use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::{AppsHostState, build_state_view, emit_system_runtime_event_to_listeners, SandboxStateView, SystemBridgeCapability, SystemRuntimeEvent};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SandboxStateChangedEvent {
    pub state: SandboxStateView,
}

impl SandboxStateChangedEvent {
    pub fn new(state: SandboxStateView) -> Self {
        Self { state }
    }
}

impl SystemRuntimeEvent for SandboxStateChangedEvent {
    const TYPE: &'static str = "sandbox.stateChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability =
        SystemBridgeCapability::SandboxListenStateChanged;
}

pub(crate) async fn emit_sandbox_state_changed(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let view = build_state_view(apps_state).await;

    emit_system_runtime_event_to_listeners(app, apps_state, SandboxStateChangedEvent::new(view))
        .await;
}
