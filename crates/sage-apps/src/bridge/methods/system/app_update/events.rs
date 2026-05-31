use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::{
    AppsHostState, SageApp, SharedSageApp, SystemBridgeCapability, SystemRuntimeEvent,
    emit_system_runtime_event_to_listeners,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub(crate) enum PendingUpdateStatusView {
    None,
    ReadyToApply,
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingUpdateChangedEvent {
    pub app_id: String,
    pub status: PendingUpdateStatusView,
}

impl SystemRuntimeEvent for PendingUpdateChangedEvent {
    const TYPE: &'static str = "appUpdate.pendingUpdateChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability = SystemBridgeCapability::AppUpdateRead;
}

pub(crate) async fn emit_pending_update_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    shared_sage_app: &SharedSageApp,
) {
    let Some((app_id, status)) = shared_sage_app.with(|sage_app| match sage_app {
        SageApp::User(user_app) => {
            let status = if user_app.pending_update().is_none() {
                PendingUpdateStatusView::None
            } else if shared_sage_app.should_review_pending_update() {
                PendingUpdateStatusView::RequiresReview
            } else {
                PendingUpdateStatusView::ReadyToApply
            };

            Some((user_app.common().id().to_string(), status))
        }
        SageApp::System(_) => None,
    }) else {
        return;
    };

    emit_system_runtime_event_to_listeners(
        app_handle,
        apps_state,
        PendingUpdateChangedEvent { app_id, status },
    )
    .await;
}
