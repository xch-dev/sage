use crate::AppsHostState;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::bridge::types::PendingBridgeApproval;
use std::collections::BTreeMap;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::bridge::registry::BridgeRegistryKind;

#[derive(Debug, Default)]
pub struct BridgeState {
    pending_approvals: Mutex<BTreeMap<String, PendingBridgeApproval>>,
}

pub(crate) async fn write_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    app_id: String,
    registry_kind: BridgeRegistryKind,
    approval: &RustBridgeApprovalRequest,
    request: &RustBridgeRequest,
) -> String {
    let approval_id = Uuid::new_v4().to_string();
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.insert(
        approval_id.to_string(),
        PendingBridgeApproval {
            approval_id: approval_id.clone(),
            app_id,
            registry_kind,
            approval: approval.clone(),
            request: request.clone(),
        },
    );

    approval_id
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

pub(crate) async fn list_pending_approvals(
    apps_state: &State<'_, AppsHostState>,
) -> Vec<PendingBridgeApproval> {
    let pending = apps_state.bridge.pending_approvals.lock().await;

    pending.values().cloned().collect()
}

pub(crate) async fn remove_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
) {
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.remove(approval_id);
}
