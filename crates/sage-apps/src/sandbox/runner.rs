use super::probes::{
    run_clear_cycle_test, run_isolation_test, run_network_test, run_persistence_test,
};
use super::state_view::{build_effective_state, build_state_view};
use super::types::{
    SandboxCapability, SandboxCapabilityStatus, SandboxRunState, SandboxState,
    build_running_sandbox_state, mark_cap,
};
use crate::AppsHostState;
use crate::utils::unix_timestamp_ms;
use tauri::{AppHandle, Manager, State};
use crate::bridge::methods::system::emit_sandbox_state_changed;

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

    let isolation_fut = run_isolation_test(&app, &apps_state);
    let persistence_fut = run_persistence_test(&app, &apps_state);
    let clear_cycle_fut = run_clear_cycle_test(&app, &apps_state);
    let network_fut = run_network_test(&app, &apps_state);

    tokio::pin!(isolation_fut);
    tokio::pin!(persistence_fut);
    tokio::pin!(clear_cycle_fut);
    tokio::pin!(network_fut);

    let mut isolation_done = false;
    let mut persistence_done = false;
    let mut clear_cycle_done = false;
    let mut network_done = false;

    while !(isolation_done && persistence_done && clear_cycle_done && network_done) {
        tokio::select! {
            res = &mut isolation_fut, if !isolation_done => {
                isolation_done = true;

                match res {
                    Ok((passed, details)) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageIsolationFromSage,
                            if passed { SandboxCapabilityStatus::Passed } else { SandboxCapabilityStatus::Failed },
                            details,
                            unix_timestamp_ms(),
                        );
                    }
                    Err(err) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageIsolationFromSage,
                            SandboxCapabilityStatus::Failed,
                            Some(err),
                            unix_timestamp_ms(),
                        );
                    }
                }

                update_current_run_state(&app, &apps_state, current_state.clone()).await;
            }

            res = &mut persistence_fut, if !persistence_done => {
                persistence_done = true;

                match res {
                    Ok((normal, incog)) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StoragePersistenceNormal,
                            if normal.0 { SandboxCapabilityStatus::Passed } else { SandboxCapabilityStatus::Failed },
                            normal.1,
                            unix_timestamp_ms(),
                        );

                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageNonPersistenceIncognito,
                            if incog.0 { SandboxCapabilityStatus::Passed } else { SandboxCapabilityStatus::Failed },
                            incog.1,
                            unix_timestamp_ms(),
                        );
                    }
                    Err(err) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StoragePersistenceNormal,
                            SandboxCapabilityStatus::Failed,
                            Some(err.clone()),
                            unix_timestamp_ms(),
                        );

                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageNonPersistenceIncognito,
                            SandboxCapabilityStatus::Failed,
                            Some(err),
                            unix_timestamp_ms(),
                        );
                    }
                }

                update_current_run_state(&app, &apps_state, current_state.clone()).await;
            }

            res = &mut clear_cycle_fut, if !clear_cycle_done => {
                clear_cycle_done = true;

                match res {
                    Ok((passed, details)) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageClearCycle,
                            if passed { SandboxCapabilityStatus::Passed } else { SandboxCapabilityStatus::Failed },
                            details,
                            unix_timestamp_ms(),
                        );
                    }
                    Err(err) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::StorageClearCycle,
                            SandboxCapabilityStatus::Failed,
                            Some(err),
                            unix_timestamp_ms(),
                        );
                    }
                }

                update_current_run_state(&app, &apps_state, current_state.clone()).await;
            }

            res = &mut network_fut, if !network_done => {
                network_done = true;

                match res {
                    Ok((passed, details)) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::NetworkAllowlistEnforced,
                            if passed { SandboxCapabilityStatus::Passed } else { SandboxCapabilityStatus::Failed },
                            details,
                            unix_timestamp_ms(),
                        );
                    }
                    Err(err) => {
                        mark_cap(
                            &mut current_state,
                            SandboxCapability::NetworkAllowlistEnforced,
                            SandboxCapabilityStatus::Failed,
                            Some(err),
                            unix_timestamp_ms(),
                        );
                    }
                }

                update_current_run_state(&app, &apps_state, current_state.clone()).await;
            }
        }
    }

    let effective = {
        let baseline = apps_state.sandbox.baseline.lock().await.clone();
        let temp_run = SandboxRunState {
            run_id: "finalize".into(),
            state: current_state.clone(),
        };
        build_effective_state(&baseline, Some(&temp_run))
    };

    current_state.overall_critical_status =
        if effective.storage_isolation_from_sage.status == SandboxCapabilityStatus::Failed {
            SandboxCapabilityStatus::Failed
        } else {
            SandboxCapabilityStatus::Passed
        };
    current_state.finished_at = Some(unix_timestamp_ms());

    *apps_state.sandbox.baseline.lock().await = current_state.clone();
    *apps_state.sandbox.current_run.lock().await = None;
    *apps_state.sandbox.running.lock().await = false;

    emit_sandbox_state_changed(&app, &apps_state).await;
}

fn sandbox_state_is_all_pending(state: &SandboxState) -> bool {
    state.storage_isolation_from_sage.status == SandboxCapabilityStatus::Pending
        && state.storage_persistence_normal.status == SandboxCapabilityStatus::Pending
        && state.storage_non_persistence_incognito.status == SandboxCapabilityStatus::Pending
        && state.storage_clear_cycle.status == SandboxCapabilityStatus::Pending
        && state.network_allowlist_enforced.status == SandboxCapabilityStatus::Pending
}
