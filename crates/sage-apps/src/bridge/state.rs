use std::collections::BTreeMap;
use std::time::Duration;

use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    AppsHostState, BridgeRegistryKind, PendingBridgeApproval, RustBridgeApprovalRequest,
    RustBridgeRequest, comms_debug, emit_bridge_approvals_changed,
    emit_timeout_for_pending_approval, sync_bridge_approval_runtime, unix_timestamp_ms,
};

const BRIDGE_APPROVAL_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Default)]
pub struct BridgeState {
    pending_approvals: Mutex<BTreeMap<String, PendingBridgeApproval>>,
    approval_expiry_task: Mutex<Option<JoinHandle<()>>>,
}

pub(crate) async fn write_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    app_id: String,
    registry_kind: BridgeRegistryKind,
    approval: &RustBridgeApprovalRequest,
    request: &RustBridgeRequest,
    approved_fingerprint: Option<u32>,
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
            approved_fingerprint,
        },
    );

    approval_id
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

/// Atomically removes and returns a pending approval under a single lock.
///
/// This is the only correct way to consume an approval before executing it:
/// a separate `get` + `remove` allows two concurrent resolves of the same
/// `approval_id` to both observe the record and double-execute the gated
/// action (e.g. `wallet.sendXch` / `wallet.getSecretKey`).
pub(crate) async fn take_pending_approval(
    apps_state: &State<'_, AppsHostState>,
    approval_id: &str,
) -> Option<PendingBridgeApproval> {
    let mut pending = apps_state.bridge.pending_approvals.lock().await;
    pending.remove(approval_id)
}

pub(crate) async fn pending_approval_app_ids(apps_state: &State<'_, AppsHostState>) -> Vec<String> {
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
    comms_debug!("approval_expiry:loop_started");

    loop {
        let apps_state: State<'_, AppsHostState> = app_handle.state();
        let pending = list_pending_approvals(&apps_state).await;

        comms_debug!("approval_expiry:tick pending={}", pending.len());

        if pending.is_empty() {
            comms_debug!("approval_expiry:empty_stop");

            let mut guard = apps_state.bridge.approval_expiry_task.lock().await;

            let pending = apps_state.bridge.pending_approvals.lock().await;
            if pending.is_empty() {
                *guard = None;
                return;
            }

            comms_debug!("approval_expiry:empty_stop_aborted_new_pending");
            continue;
        }

        let now = unix_timestamp_ms() as u64;
        let mut next_expiry: Option<u64> = None;
        let mut expired = Vec::new();

        for approval in pending {
            comms_debug!(
                "approval_expiry:check id={} app={} now={} expires_at={} remaining_ms={}",
                approval.approval_id,
                approval.app_id,
                now,
                approval.expires_at_ms,
                approval.expires_at_ms.saturating_sub(now),
            );

            if approval.expires_at_ms <= now {
                expired.push(approval);
            } else {
                next_expiry = Some(match next_expiry {
                    Some(current) => current.min(approval.expires_at_ms),
                    None => approval.expires_at_ms,
                });
            }
        }

        comms_debug!("approval_expiry:expired count={}", expired.len());

        for approval in &expired {
            comms_debug!(
                "approval_expiry:remove id={} app={}",
                approval.approval_id,
                approval.app_id,
            );

            remove_pending_approval(&apps_state, &approval.approval_id).await;

            if let Err(err) =
                emit_timeout_for_pending_approval(&app_handle, &apps_state, approval).await
            {
                comms_debug!(
                    "approval_expiry:timeout_emit_failed id={} error={}",
                    approval.approval_id,
                    err,
                );
            }
        }

        if !expired.is_empty() {
            comms_debug!("approval_expiry:sync_after_expired");

            if let Err(err) = sync_bridge_approval_runtime(&app_handle, &apps_state).await {
                comms_debug!("approval_expiry:sync_failed error={}", err);
            }

            emit_bridge_approvals_changed(&app_handle, &apps_state).await;
        }

        let Some(next_expiry) = next_expiry else {
            comms_debug!("approval_expiry:no_next_continue");
            continue;
        };

        if next_expiry > now {
            let sleep_ms = next_expiry - now;
            comms_debug!("approval_expiry:sleep ms={}", sleep_ms);
            tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    }
}
