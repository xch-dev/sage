use tauri::State;
use tokio::time::{Duration, sleep};

use super::super::store::SandboxAppResult;
use super::super::types::{
    SandboxIsolationProbeResult, SandboxNetworkProbeResult, SandboxPersistenceReadProbeResult,
    SandboxPersistenceWriteProbeResult, SandboxStorageClearProbePhase,
    SandboxStorageClearProbeResult,
};
use crate::state::AppsHostState;
use crate::utils::unix_timestamp_ms;

pub async fn poll_isolation(
    apps_state: &State<'_, AppsHostState>,
    run_id: &str,
    expected_count: usize,
    timeout_ms: i64,
) -> Result<Vec<SandboxAppResult<SandboxIsolationProbeResult>>, String> {
    let started = unix_timestamp_ms();

    loop {
        let results = {
            let runs = apps_state.sandbox.runs.lock().await;
            runs.get(run_id)
                .map(|r| r.isolation.clone())
                .unwrap_or_default()
        };

        if results.len() >= expected_count {
            return Ok(results);
        }

        if unix_timestamp_ms() - started >= timeout_ms {
            return Err("Timed out waiting for sandbox isolation results.".into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn poll_persistence_write(
    apps_state: &State<'_, AppsHostState>,
    run_id: &str,
    expected_count: usize,
    timeout_ms: i64,
) -> Result<Vec<SandboxAppResult<SandboxPersistenceWriteProbeResult>>, String> {
    let started = unix_timestamp_ms();

    loop {
        let results = {
            let runs = apps_state.sandbox.runs.lock().await;
            runs.get(run_id)
                .map(|r| r.persistence_write.clone())
                .unwrap_or_default()
        };

        if results.len() >= expected_count {
            return Ok(results);
        }

        if unix_timestamp_ms() - started >= timeout_ms {
            return Err("Timed out waiting for sandbox persistence write results.".into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn poll_persistence_read(
    apps_state: &State<'_, AppsHostState>,
    run_id: &str,
    expected_count: usize,
    timeout_ms: i64,
) -> Result<Vec<SandboxAppResult<SandboxPersistenceReadProbeResult>>, String> {
    let started = unix_timestamp_ms();

    loop {
        let results = {
            let runs = apps_state.sandbox.runs.lock().await;
            runs.get(run_id)
                .map(|r| r.persistence_read.clone())
                .unwrap_or_default()
        };

        if results.len() >= expected_count {
            return Ok(results);
        }

        if unix_timestamp_ms() - started >= timeout_ms {
            return Err("Timed out waiting for sandbox persistence read results.".into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn poll_network(
    apps_state: &State<'_, AppsHostState>,
    run_id: &str,
    expected_count: usize,
    timeout_ms: i64,
) -> Result<Vec<SandboxAppResult<SandboxNetworkProbeResult>>, String> {
    let started = unix_timestamp_ms();

    loop {
        let results = {
            let runs = apps_state.sandbox.runs.lock().await;
            runs.get(run_id)
                .map(|r| r.network.clone())
                .unwrap_or_default()
        };

        if results.len() >= expected_count {
            return Ok(results);
        }

        if unix_timestamp_ms() - started >= timeout_ms {
            return Err("Timed out waiting for sandbox network results.".into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}

pub async fn poll_clear_cycle_phase(
    apps_state: &State<'_, AppsHostState>,
    run_id: &str,
    app_id: &str,
    phase: SandboxStorageClearProbePhase,
    timeout_ms: i64,
) -> Result<SandboxStorageClearProbeResult, String> {
    let started = unix_timestamp_ms();

    loop {
        let results = {
            let runs = apps_state.sandbox.runs.lock().await;
            runs.get(run_id)
                .map(|r| r.clear_cycle.clone())
                .unwrap_or_default()
        };

        if let Some(found) = results
            .into_iter()
            .find(|item| item.app_id == app_id && item.data.phase == phase)
        {
            return Ok(found.data);
        }

        if unix_timestamp_ms() - started >= timeout_ms {
            return Err("Timed out waiting for sandbox storage clear phase.".into());
        }

        sleep(Duration::from_millis(100)).await;
    }
}
