use crate::AppsHostState;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::bridge::types::PendingBridgeApproval;
use std::collections::BTreeMap;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tauri::async_runtime::JoinHandle;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::bridge::registry::BridgeRegistryKind;
use crate::runtime::{emit_bridge_approvals_changed, emit_timeout_for_pending_approval, sync_bridge_approval_runtime};
use crate::utils::unix_timestamp_ms;

const BRIDGE_APPROVAL_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Default)]
pub struct BridgeState {
    pending_approvals: Mutex<BTreeMap<String, PendingBridgeApproval>>,
    approval_expiry_task: Mutex<Option<JoinHandle<()>>>
}

pub(crate) async fn write_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    app_id: String,
    registry_kind: BridgeRegistryKind,
    approval: &RustBridgeApprovalRequest,
    request: &RustBridgeRequest,
) -> String {
    let approval_id = Uuid::new_v4().to_string();
    let now = unix_timestamp_ms() as u64;
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.insert(
        approval_id.to_string(),
        PendingBridgeApproval {
            approval_id: approval_id.clone(),
            app_id,
            registry_kind,
            approval: approval.clone(),
            request: request.clone(),
            created_at_ms: now,
            expires_at_ms: now + BRIDGE_APPROVAL_TIMEOUT_MS,
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

pub(crate) async fn pending_approval_app_ids(
    apps_state: &State<'_, AppsHostState>,
) -> Vec<String> {
    use std::collections::BTreeSet;

    list_pending_approvals(apps_state)
        .await
        .into_iter()
        .map(|approval| approval.app_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub async fn ensure_approval_expiry_loop(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let mut guard = apps_state.bridge.approval_expiry_task.lock().await;

    if guard.is_some() {
        return;
    }

    let handle = {
        let app_handle = app_handle.clone();

        tauri::async_runtime::spawn(async move {
            approval_expiry_loop(app_handle).await;
        })
    };

    *guard = Some(handle);
}

async fn approval_expiry_loop(app_handle: AppHandle) {
    loop {
        let apps_state: State<'_, AppsHostState> = app_handle.state();

        let pending = list_pending_approvals(&apps_state).await;

        if pending.is_empty() {
            let mut guard = apps_state.bridge.approval_expiry_task.lock().await;
            *guard = None;
            return;
        }

        let now = unix_timestamp_ms() as u64;

        let mut next_expiry: Option<u64> = None;
        let mut expired = Vec::new();

        for approval in pending {
            if approval.expires_at_ms <= now {
                expired.push(approval);
            } else {
                next_expiry = Some(match next_expiry {
                    Some(current) => current.min(approval.expires_at_ms),
                    None => approval.expires_at_ms,
                });
            }
        }

        for approval in &expired {
            remove_pending_approval(&apps_state, &approval.approval_id).await;
            let _ = emit_timeout_for_pending_approval(&app_handle, &apps_state, approval).await;
        }

        if !expired.is_empty() {
            let _ = sync_bridge_approval_runtime(&app_handle, &apps_state).await;
            emit_bridge_approvals_changed(&app_handle, &apps_state).await;
        }

        let Some(next_expiry) = next_expiry else {
            continue;
        };

        if next_expiry > now {
            tokio::time::sleep(Duration::from_millis(next_expiry - now)).await;
        }
    }
}
