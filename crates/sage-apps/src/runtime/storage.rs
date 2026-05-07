use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tokio::time::sleep;
use crate::AppsHostState;
use crate::lifecycle::{clear_app_storage_by_target};
use crate::runtime::{SageAppRuntimeImpostorKind};
use crate::runtime::start::{create_impostor_runtime_from_stopped, CreateImpostorRuntimeArgs};
use crate::runtime::stop::close_runtime_internal;
use crate::sandbox::{build_builtin_runtime_app, SandboxStorageClearProbePhase, SandboxStorageClearProbeResult, BUILTIN_STORAGE_CLEAR_PROBE_RUNTIME_ID};
use crate::storage::cleanup_target_from_storage;
use crate::types::SharedSageApp;
use crate::utils::unix_timestamp_ms;

pub(crate) async fn run_verified_storage_clear_cycle(
    app_handle: &AppHandle,
    resolved_app: &crate::types::ResolvedStoppedApp,
) -> Result<(), String> {
    let apps_state: State<'_, AppsHostState> = app_handle.state();

    let app_id = resolved_app.with_app(SharedSageApp::id);
    let run_id = unique_run_id("storage-clear-cycle");

    let write = run_storage_clear_phase(
        app_handle,
        &apps_state,
        resolved_app,
        &run_id,
        SandboxStorageClearProbePhase::Write,
    )
        .await?;

    if write.error.is_some() || !write.local_storage_present || !write.indexed_db_present {
        return Err(write
            .error
            .unwrap_or_else(|| "storage clear write probe failed".into()));
    }

    let present = run_storage_clear_phase(
        app_handle,
        &apps_state,
        resolved_app,
        &run_id,
        SandboxStorageClearProbePhase::CheckPresent,
    )
        .await?;

    if present.error.is_some() || !present.local_storage_present || !present.indexed_db_present {
        return Err(present
            .error
            .unwrap_or_else(|| "storage clear presence probe failed".into()));
    }

    let storage = resolved_app.with_app(|app| {
        app.with(|app| app.storage().clone())
    });

    let target = cleanup_target_from_storage(&storage);
    clear_app_storage_by_target(app_handle, &target).await?;

    let absent = run_storage_clear_phase(
        app_handle,
        &apps_state,
        resolved_app,
        &run_id,
        SandboxStorageClearProbePhase::CheckAbsent,
    )
        .await?;

    if let Some(error) = absent.error {
        return Err(error);
    }

    if absent.local_storage_present || absent.indexed_db_present {
        return Err("storage clear verification failed because probe data was still visible".into());
    }

    resolved_app
        .try_with_app(|app| {
            app.try_mutate(|app| {
                app.common_mut().clear_storage_may_contain_secrets();

                Ok::<(), anyhow::Error>(())
            })
        })
        .map_err(|err| format!("failed to persist cleared storage state: {err}"))?;

    close_runtime_internal(app_handle, &apps_state, &app_id).await;

    Ok(())
}

async fn run_storage_clear_phase(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    resolved_app: &crate::types::ResolvedStoppedApp,
    run_id: &str,
    phase: SandboxStorageClearProbePhase,
) -> Result<SandboxStorageClearProbeResult, String> {
    let app_id = resolved_app.with_app(SharedSageApp::id);

    let phase_string = match phase {
        SandboxStorageClearProbePhase::Write => "write",
        SandboxStorageClearProbePhase::CheckPresent => "check_present",
        SandboxStorageClearProbePhase::CheckAbsent => "check_absent",
    }
        .to_string();

    let impostor_app = build_builtin_runtime_app(BUILTIN_STORAGE_CLEAR_PROBE_RUNTIME_ID)
        .map_err(|err| format!("failed to build storage clear probe runtime app: {err}"))?
        .ok_or_else(|| "missing storage clear probe runtime app".to_string())?;

    let impostor_app = SharedSageApp::new(impostor_app);

    let mut query = std::collections::BTreeMap::new();
    query.insert("runId".to_string(), run_id.to_string());
    query.insert("phase".to_string(), phase_string);
    query.insert("appId".to_string(), app_id.clone());

    create_impostor_runtime_from_stopped(
        app_handle.clone(),
        apps_state.clone(),
        resolved_app,
        impostor_app,
        CreateImpostorRuntimeArgs {
            kind: SageAppRuntimeImpostorKind::StorageClearProbe,
            debug_layout: false,
            query,
        },
    )
        .await?;

    let out = poll_clear_cycle_phase(apps_state, run_id, &app_id, phase, 10_000).await;

    close_runtime_internal(app_handle, apps_state, &app_id).await;

    out
}

fn unique_run_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

async fn poll_clear_cycle_phase(
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
