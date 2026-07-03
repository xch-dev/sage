use tauri::State;

use super::types::{
    SandboxCapabilityResult, SandboxRunState, SandboxState, SandboxStateView,
};
use crate::AppsHostState;

fn effective_cap(
    _baseline: &SandboxCapabilityResult,
    current: &SandboxCapabilityResult,
) -> SandboxCapabilityResult {
    // While a run is in progress we must reflect the *live* status of each
    // capability, never the previous baseline. Falling back to a stale
    // (previously `Passed`) baseline for a `Pending`/`Running` capability let a
    // user-triggered rerun keep the launch gate open using results that no
    // longer apply — and a runner that crashed mid-rerun could leave apps
    // launchable indefinitely. Surfacing the live `Running`/`Pending` status
    // makes the gate block until the rerun actually re-establishes each result.
    current.clone()
}

pub fn build_effective_state(
    baseline: &SandboxState,
    current_run: Option<&SandboxRunState>,
) -> SandboxState {
    let Some(current_run) = current_run else {
        return baseline.clone();
    };

    let current = &current_run.state;

    SandboxState {
        overall_critical_status: baseline.overall_critical_status,
        storage_isolation_from_sage: effective_cap(
            &baseline.storage_isolation_from_sage,
            &current.storage_isolation_from_sage,
        ),
        storage_persistence_normal: effective_cap(
            &baseline.storage_persistence_normal,
            &current.storage_persistence_normal,
        ),
        storage_non_persistence_incognito: effective_cap(
            &baseline.storage_non_persistence_incognito,
            &current.storage_non_persistence_incognito,
        ),
        network_allowlist_enforced: effective_cap(
            &baseline.network_allowlist_enforced,
            &current.network_allowlist_enforced,
        ),
        started_at: baseline.started_at,
        finished_at: baseline.finished_at,
    }
}

pub async fn build_state_view(apps_state: &State<'_, AppsHostState>) -> SandboxStateView {
    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();

    let effective = build_effective_state(&baseline, current_run.as_ref());

    SandboxStateView {
        baseline,
        current_run,
        effective,
    }
}
