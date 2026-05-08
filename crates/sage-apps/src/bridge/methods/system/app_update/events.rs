use crate::AppsHostState;
use crate::bridge::emit_system_runtime_event_to_listeners;
use crate::bridge::event_emit::SystemRuntimeEvent;
use crate::capabilities::list::SystemBridgeCapability;
use crate::types::{SageApp, SharedSageApp, UserSageAppView};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingUpdateChangedEvent {
    pub app_id: String,
    pub app: UserSageAppView,
}

impl SystemRuntimeEvent for PendingUpdateChangedEvent {
    const TYPE: &'static str = "appUpdate.pendingUpdateChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability = SystemBridgeCapability::AppUpdateRead;
}

pub(crate) async fn emit_pending_update_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app: &SharedSageApp,
) {
    let Some((app_id, app)) = app.with(|app| match app {
        SageApp::User(user_app) => {
            Some((user_app.common().id().to_string(), UserSageAppView::from(user_app)))
        }
        SageApp::System(_) => None,
    }) else {
        return;
    };

    emit_system_runtime_event_to_listeners(
        app_handle,
        apps_state,
        PendingUpdateChangedEvent { app_id, app },
    )
        .await;
}
