use tauri::{AppHandle, Manager, State};

use super::probes::{run_isolation_test, run_network_test, run_persistence_test};
use super::state_view::{build_effective_state, build_state_view};
use super::types::{
    SandboxCapability, SandboxCapabilityStatus, SandboxRunState, SandboxState,
    build_running_sandbox_state, mark_cap,
};
use crate::{AppsHostState, emit_sandbox_state_changed, unix_timestamp_ms};

pub async fn ensure_initial_sandbox_run(app: AppHandle) -> Result<(), String> {
    let apps_state = app.state::<AppsHostState>();

    let already_running = *apps_state.sandbox.running.lock().await;
    if already_running {
        return Ok(());
    }

    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();

    if current_run.is_some() {
        return Ok(());
    }

    if !sandbox_state_is_all_pending(&baseline) {
        return Ok(());
    }

    begin_sandbox_run(&app, &apps_state).await?;

    let runner_app = app.clone();
    tokio::spawn(async move {
        let runner = Box::pin(sandbox_runner(runner_app));
        runner.await;
    });

    Ok(())
}

pub(crate) async fn begin_sandbox_run(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<super::types::SandboxStateView, String> {
    {
        let mut running = apps_state.sandbox.running.lock().await;
        if *running {
            return Ok(build_state_view(apps_state).await);
        }
        *running = true;
    }

    {
        let mut runs = apps_state.sandbox.runs.lock().await;
        runs.clear();
    }

    let run_state = SandboxRunState {
        run_id: super::runtime::unique_run_id("sandbox-run"),
        state: build_running_sandbox_state(unix_timestamp_ms()),
    };

    *apps_state.sandbox.current_run.lock().await = Some(run_state);

    emit_sandbox_state_changed(app, apps_state).await;

    Ok(build_state_view(apps_state).await)
}

async fn update_current_run_state(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    state: SandboxState,
) {
    if let Some(current_run) = apps_state.sandbox.current_run.lock().await.as_mut() {
        current_run.state = state;
    }
    emit_sandbox_state_changed(app, apps_state).await;
}

pub async fn sandbox_runner(app: AppHandle) {
    let apps_state = app.state::<AppsHostState>();

    let mut current_state = {
        let current_run = apps_state.sandbox.current_run.lock().await.clone();

        current_run.map_or_else(
            || build_running_sandbox_state(unix_timestamp_ms()),
            |r| r.state,
        )
    };

    mark_running(
        &mut current_state,
        &[SandboxCapability::NetworkAllowlistEnforced],
    );
    update_current_run_state(&app, &apps_state, current_state.clone()).await;

    // Storage probes share WebView lifecycle concerns and must not overlap each
    // other. The network probe is independent, so keep it in a parallel lane.
    let ((), network_result) = tokio::join!(
        async {
            record_single_probe_result(
                &mut current_state,
                SandboxCapability::StorageIsolationFromSage,
                run_isolation_test(&app, &apps_state).await,
            );
            mark_running(
                &mut current_state,
                &[
                    SandboxCapability::StoragePersistenceNormal,
                    SandboxCapability::StorageNonPersistenceIncognito,
                ],
            );
            update_current_run_state(&app, &apps_state, current_state.clone()).await;

            record_persistence_probe_result(
                &mut current_state,
                run_persistence_test(&app, &apps_state).await,
            );
            update_current_run_state(&app, &apps_state, current_state.clone()).await;
        },
        run_network_test(&app, &apps_state),
    );

    record_single_probe_result(
        &mut current_state,
        SandboxCapability::NetworkAllowlistEnforced,
        network_result,
    );
    update_current_run_state(&app, &apps_state, current_state.clone()).await;

    let effective = {
        let baseline = apps_state.sandbox.baseline.lock().await.clone();
        let temp_run = SandboxRunState {
            run_id: "finalize".into(),
            state: current_state.clone(),
        };
        build_effective_state(&baseline, Some(&temp_run))
    };

    current_state.overall_critical_status = overall_status(&effective);
    current_state.finished_at = Some(unix_timestamp_ms());

    *apps_state.sandbox.baseline.lock().await = current_state.clone();
    *apps_state.sandbox.current_run.lock().await = None;
    *apps_state.sandbox.running.lock().await = false;

    emit_sandbox_state_changed(&app, &apps_state).await;
}

fn mark_running(state: &mut SandboxState, capabilities: &[SandboxCapability]) {
    for capability in capabilities {
        mark_cap(
            state,
            *capability,
            SandboxCapabilityStatus::Running,
            None,
            unix_timestamp_ms(),
        );
    }
}

fn record_single_probe_result(
    state: &mut SandboxState,
    capability: SandboxCapability,
    result: Result<(bool, Option<String>), String>,
) {
    let (status, details) = match result {
        Ok((passed, details)) => (
            if passed {
                SandboxCapabilityStatus::Passed
            } else {
                SandboxCapabilityStatus::Failed
            },
            details,
        ),
        Err(err) => (SandboxCapabilityStatus::Failed, Some(err)),
    };

    mark_cap(state, capability, status, details, unix_timestamp_ms());
}

type PersistenceProbeResult = Result<((bool, Option<String>), (bool, Option<String>)), String>;

fn record_persistence_probe_result(state: &mut SandboxState, result: PersistenceProbeResult) {
    match result {
        Ok((normal, incognito)) => {
            record_single_probe_result(
                state,
                SandboxCapability::StoragePersistenceNormal,
                Ok(normal),
            );
            record_single_probe_result(
                state,
                SandboxCapability::StorageNonPersistenceIncognito,
                Ok(incognito),
            );
        }
        Err(err) => {
            record_single_probe_result(
                state,
                SandboxCapability::StoragePersistenceNormal,
                Err(err.clone()),
            );
            record_single_probe_result(
                state,
                SandboxCapability::StorageNonPersistenceIncognito,
                Err(err),
            );
        }
    }
}

fn overall_status(state: &SandboxState) -> SandboxCapabilityStatus {
    let statuses = [
        state.storage_isolation_from_sage.status,
        state.storage_persistence_normal.status,
        state.storage_non_persistence_incognito.status,
        state.network_allowlist_enforced.status,
    ];

    if statuses.contains(&SandboxCapabilityStatus::Failed) {
        SandboxCapabilityStatus::Failed
    } else if statuses.contains(&SandboxCapabilityStatus::Running) {
        SandboxCapabilityStatus::Running
    } else if statuses.contains(&SandboxCapabilityStatus::Pending) {
        SandboxCapabilityStatus::Pending
    } else {
        SandboxCapabilityStatus::Passed
    }
}

fn sandbox_state_is_all_pending(state: &SandboxState) -> bool {
    state.storage_isolation_from_sage.status == SandboxCapabilityStatus::Pending
        && state.storage_persistence_normal.status == SandboxCapabilityStatus::Pending
        && state.storage_non_persistence_incognito.status == SandboxCapabilityStatus::Pending
        && state.network_allowlist_enforced.status == SandboxCapabilityStatus::Pending
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::types::{build_initial_sandbox_state, mark_cap};

    #[test]
    fn overall_status_fails_when_any_capability_fails() {
        let mut state = build_initial_sandbox_state();

        for capability in [
            SandboxCapability::StorageIsolationFromSage,
            SandboxCapability::StoragePersistenceNormal,
            SandboxCapability::StorageNonPersistenceIncognito,
            SandboxCapability::NetworkAllowlistEnforced,
        ] {
            mark_cap(
                &mut state,
                capability,
                SandboxCapabilityStatus::Passed,
                None,
                1,
            );
        }

        mark_cap(
            &mut state,
            SandboxCapability::StorageNonPersistenceIncognito,
            SandboxCapabilityStatus::Failed,
            None,
            2,
        );

        assert_eq!(overall_status(&state), SandboxCapabilityStatus::Failed);
    }
}
