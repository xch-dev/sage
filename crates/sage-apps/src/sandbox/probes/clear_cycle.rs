use crate::AppsHostState;
use crate::lifecycle::clear_runtime_browsing_data_internal;
use crate::runtime::stop::close_runtime_internal;
use crate::sandbox::BUILTIN_STORAGE_CLEAR_PERSISTENT_ID;
use tauri::{AppHandle, State};

use super::super::runtime::{run_clear_cycle_phase_runtime, unique_run_id};
use super::super::types::{SandboxStorageClearProbePhase, SandboxStorageClearProbeResult};
use super::poll::poll_clear_cycle_phase;

pub(in crate::sandbox) async fn run_clear_cycle_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let run_id = unique_run_id("storage-clear-cycle");
    let app_id = BUILTIN_STORAGE_CLEAR_PERSISTENT_ID;

    let write = run_clear_cycle_phase(
        app,
        apps_state,
        app_id,
        &run_id,
        SandboxStorageClearProbePhase::Write,
    )
    .await?;

    if write.error.is_some() || !write.local_storage_present || !write.indexed_db_present {
        return Ok((
            false,
            Some(
                write
                    .error
                    .unwrap_or_else(|| "Storage clear write probe failed.".into()),
            ),
        ));
    }

    let present = run_clear_cycle_phase(
        app,
        apps_state,
        app_id,
        &run_id,
        SandboxStorageClearProbePhase::CheckPresent,
    )
    .await?;

    if present.error.is_some() || !present.local_storage_present || !present.indexed_db_present {
        return Ok((
            false,
            Some(
                present
                    .error
                    .unwrap_or_else(|| "Storage clear presence probe failed.".into()),
            ),
        ));
    }

    clear_runtime_browsing_data_internal(app, apps_state, app_id).await?;

    let absent = run_clear_cycle_phase(
        app,
        apps_state,
        app_id,
        &run_id,
        SandboxStorageClearProbePhase::CheckAbsent,
    )
    .await?;

    if absent.error.is_some() {
        return Ok((false, absent.error));
    }

    let passed = !absent.local_storage_present && !absent.indexed_db_present;

    Ok((
        passed,
        Some(if passed {
            "Storage clear cycle removed localStorage and IndexedDB for the target app origin."
                .into()
        } else {
            "Storage clear cycle failed because data was still visible after host-side clearing."
                .into()
        }),
    ))
}

async fn run_clear_cycle_phase(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    run_id: &str,
    phase: SandboxStorageClearProbePhase,
) -> Result<SandboxStorageClearProbeResult, String> {
    let phase_string = match phase {
        SandboxStorageClearProbePhase::Write => "write",
        SandboxStorageClearProbePhase::CheckPresent => "check_present",
        SandboxStorageClearProbePhase::CheckAbsent => "check_absent",
    }
    .to_string();

    run_clear_cycle_phase_runtime(app, apps_state, app_id, run_id, phase_string).await?;

    let out = poll_clear_cycle_phase(apps_state, run_id, app_id, phase, 10_000).await;

    let _ = close_runtime_internal(app, apps_state, app_id).await;

    out
}
