use crate::AppsHostState;
use crate::bridge::RustBridgeRequest;
use crate::bridge::types::PendingBridgeApproval;
use crate::types::{SharedSageApp};
use std::collections::BTreeMap;
use tauri::State;
use tokio::sync::Mutex;
use crate::bridge::registry::BridgeRegistryKind;

#[derive(Debug, Default)]
pub struct BridgeState {
    pending_approvals: Mutex<BTreeMap<String, PendingBridgeApproval>>,
}

pub(crate) async fn write_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
    sage_app: &SharedSageApp,
    request: &RustBridgeRequest,
    registry_kind: BridgeRegistryKind,
) {
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.insert(
        approval_id.to_string(),
        PendingBridgeApproval {
            app_webview_label: sage_app.webview_label(),
            request: request.clone(),
            registry_kind
        },
    );
}

pub(crate) async fn find_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
) -> Option<PendingBridgeApproval> {
    let pending = apps_state.bridge.pending_approvals.lock().await;
    pending.get(approval_id).cloned()
}

pub(crate) async fn get_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
) -> Result<PendingBridgeApproval, String> {
    find_pending_approval(apps_state, approval_id)
        .await
        .ok_or_else(|| format!("No pending approval with id {approval_id}"))
}

pub(crate) async fn remove_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
) {
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.remove(approval_id);
}
