use tauri::{AppHandle, Manager, State};
use tokio::time::{Duration, sleep};

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

    // On a loaded machine: the
    // first run starts at wallet login, the same moment the wallet subscribes to its puzzle
    // hashes and its database is busiest, and a probe that merely could not FINISH — a test
    // webview slow to open or close, a report that missed its window, example.com slow to
    // answer — failed the capability, and with it every app, for the rest of the session.
    // Such a run is tried again (PROBE_ATTEMPTS in all). A probe that OBSERVED the sandbox
    // doing the wrong thing is final the first time: no retry can wash it out.
    for attempt in 1..=PROBE_ATTEMPTS {
        let outcome = run_probes_once(&app, &apps_state, &mut current_state).await;
        if !should_retry(outcome, attempt) {
            break;
        }
        tracing::warn!(
            "sandbox: a probe could not finish (attempt {attempt} of {PROBE_ATTEMPTS}); running the probes again in {} s",
            RETRY_PAUSE.as_secs()
        );
        sleep(RETRY_PAUSE).await;
    }

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

///  how many times one sandbox run tries the probes when one could not finish.
pub(crate) const PROBE_ATTEMPTS: usize = 3;
const RETRY_PAUSE: Duration = Duration::from_secs(5);

/// What one pass of the probes came to, for the retry decision.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ProbeOutcome {
    /// A probe could not finish (it returned `Err`: a timeout, a webview that would not open
    /// or close, the allowed URL out of reach).
    pub(crate) unfinished: bool,
    /// A probe finished and saw the sandbox do the wrong thing (`Ok(false)`).
    pub(crate) violation: bool,
}

impl ProbeOutcome {
    fn note(&mut self, result: &Result<(bool, Option<String>), String>) {
        match result {
            Err(_) => self.unfinished = true,
            Ok((false, _)) => self.violation = true,
            Ok((true, _)) => {}
        }
    }
}

/// Try again only when something could not finish, nothing was seen to fail, and attempts remain.
pub(crate) fn should_retry(outcome: ProbeOutcome, attempt: usize) -> bool {
    outcome.unfinished && !outcome.violation && attempt < PROBE_ATTEMPTS
}

async fn run_probes_once(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    current_state: &mut SandboxState,
) -> ProbeOutcome {
    let mut outcome = ProbeOutcome::default();

    mark_running(
        current_state,
        &[
            SandboxCapability::StorageIsolationFromSage,
            SandboxCapability::NetworkAllowlistEnforced,
        ],
    );
    update_current_run_state(app, apps_state, current_state.clone()).await;

    // Storage probes share WebView lifecycle concerns and must not overlap each
    // other. The network probe is independent, so keep it in a parallel lane.
    let ((), network_result) = tokio::join!(
        async {
            let isolation = run_isolation_test(app, apps_state).await;
            outcome.note(&isolation);
            record_single_probe_result(
                current_state,
                SandboxCapability::StorageIsolationFromSage,
                isolation,
            );
            mark_running(
                current_state,
                &[
                    SandboxCapability::StoragePersistenceNormal,
                    SandboxCapability::StorageNonPersistenceIncognito,
                ],
            );
            update_current_run_state(app, apps_state, current_state.clone()).await;

            let persistence = run_persistence_test(app, apps_state).await;
            match &persistence {
                Ok((normal, incognito)) => {
                    outcome.note(&Ok(normal.clone()));
                    outcome.note(&Ok(incognito.clone()));
                }
                Err(err) => outcome.note(&Err(err.clone())),
            }
            record_persistence_probe_result(current_state, persistence);
            update_current_run_state(app, apps_state, current_state.clone()).await;
        },
        run_network_test(app, apps_state),
    );

    outcome.note(&network_result);
    record_single_probe_result(
        current_state,
        SandboxCapability::NetworkAllowlistEnforced,
        network_result,
    );
    update_current_run_state(app, apps_state, current_state.clone()).await;

    outcome
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

    if matches!(status, SandboxCapabilityStatus::Passed) {
        tracing::info!(
            "sandbox {capability:?}: passed ({})",
            details.as_deref().unwrap_or("")
        );
    } else {
        tracing::warn!(
            "sandbox {capability:?}: {status:?} ({})",
            details.as_deref().unwrap_or("")
        );
    }

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
    fn retries_only_an_unfinished_run_with_no_violation_while_attempts_remain() {
        let unfinished = ProbeOutcome {
            unfinished: true,
            violation: false,
        };
        let clean = ProbeOutcome::default();
        let violated = ProbeOutcome {
            unfinished: false,
            violation: true,
        };
        let both = ProbeOutcome {
            unfinished: true,
            violation: true,
        };

        assert!(should_retry(unfinished, 1));
        assert!(should_retry(unfinished, PROBE_ATTEMPTS - 1));
        assert!(
            !should_retry(unfinished, PROBE_ATTEMPTS),
            "the last attempt is final"
        );
        assert!(!should_retry(clean, 1), "a clean pass is not run again");
        assert!(
            !should_retry(violated, 1),
            "a seen violation is never retried"
        );
        assert!(
            !should_retry(both, 1),
            "a violation beside a timeout is still never retried"
        );
    }

    #[test]
    fn outcome_notes_err_as_unfinished_and_false_as_violation() {
        let mut o = ProbeOutcome::default();
        o.note(&Ok((true, None)));
        assert_eq!(o, ProbeOutcome::default());
        o.note(&Err("timed out".into()));
        assert!(o.unfinished && !o.violation);
        o.note(&Ok((false, Some("saw it".into()))));
        assert!(o.violation);
    }

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
