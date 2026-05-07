use std::cmp::Reverse;
use std::fmt::Display;
use std::time::{Duration, Instant};

use tauri::State;

use crate::AppsHostState;
use crate::runtime::SageAppRuntimeVisibility;
use crate::runtime::state::types::{SharedImpostorRuntime, SharedRuntime};
use crate::types::AppPresentation;

const IMMEDIATE_LOCK_RETRY_TIMEOUT_MS: u64 = 20;
const IMMEDIATE_LOCK_RETRY_DELAY_MS: u64 = 2;

pub enum GetRuntimeError {
    NotFound,
}

impl Display for GetRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GetRuntimeError::NotFound => String::from("Runtime not found"),
            },
        )
    }
}

pub async fn find_runtime_by_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Option<SharedRuntime> {
    let runtime_id = find_runtime_id_by_app_id_optional(apps_state, app_id).await?;
    find_runtime_by_runtime_id_optional(apps_state, &runtime_id).await
}

pub(crate) fn find_runtime_by_app_id_optional_immediate(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<SharedRuntime>, String> {
    retry_immediate_lookup("runtime lookup", || {
        find_runtime_by_app_id_optional_immediate_once(apps_state, app_id)
    })
}

pub(crate) async fn find_runtime_id_by_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Option<String> {
    let by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
    by_app_id.get(app_id).cloned()
}

pub(crate) async fn find_runtime_by_runtime_id_optional(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) -> Option<SharedRuntime> {
    let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
    by_runtime_id.get(runtime_id).cloned()
}

pub(crate) async fn find_impostor_runtime_by_victim_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    victim_app_id: &str,
) -> Option<SharedImpostorRuntime> {
    let runtime_id =
        find_impostor_runtime_id_by_victim_app_id_optional(apps_state, victim_app_id).await?;

    find_impostor_runtime_by_runtime_id_optional(apps_state, &runtime_id).await
}

pub(crate) fn find_impostor_runtime_by_victim_app_id_optional_immediate(
    apps_state: &State<'_, AppsHostState>,
    victim_app_id: &str,
) -> Result<Option<SharedImpostorRuntime>, String> {
    retry_immediate_lookup("impostor runtime lookup", || {
        find_impostor_runtime_by_victim_app_id_optional_immediate_once(
            apps_state,
            victim_app_id,
        )
    })
}

pub(crate) async fn find_impostor_runtime_id_by_victim_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    victim_app_id: &str,
) -> Option<String> {
    let by_victim_app_id = apps_state
        .runtime
        .impostor_runtime_id_by_victim_app_id
        .lock()
        .await;

    by_victim_app_id.get(victim_app_id).cloned()
}

pub(crate) async fn find_impostor_runtime_by_runtime_id_optional(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) -> Option<SharedImpostorRuntime> {
    let by_runtime_id = apps_state.runtime.impostor_by_runtime_id.lock().await;
    by_runtime_id.get(runtime_id).cloned()
}

pub(crate) async fn get_runtime_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, GetRuntimeError> {
    find_runtime_by_app_id_optional(apps_state, app_id)
        .await
        .ok_or(GetRuntimeError::NotFound)
}

pub(crate) async fn list_runtimes(
    apps_state: &State<'_, AppsHostState>,
) -> Result<Vec<SharedRuntime>, String> {
    let mut runtimes = {
        let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.values().cloned().collect::<Vec<_>>()
    };

    runtimes.retain(|runtime| {
        !runtime.with_runtime(super::types::SageAppRuntimeRecord::internal)
    });

    runtimes.sort_by_key(|runtime| {
        Reverse(runtime.with_runtime(super::types::SageAppRuntimeRecord::started_at))
    });

    Ok(runtimes)
}

pub(crate) async fn find_active_taskbar_runtime(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
) -> Option<SharedRuntime> {
    let taskbar_runtimes = get_taskbar_runtimes(apps_state, host_window_label).await;
    taskbar_runtimes.iter().find(|runtime| runtime.with_runtime(
        |runtime|
            runtime.visibility() == SageAppRuntimeVisibility::Visible
    )).cloned()
}

pub(in crate::runtime) async fn is_apps_workspace_active(
    apps_state: &State<'_, AppsHostState>,
) -> bool {
    *apps_state.runtime.apps_workspace_active.read().await
}

async fn get_taskbar_runtimes(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
) -> Vec<SharedRuntime> {
    let runtimes: Vec<SharedRuntime> = {
        apps_state
            .runtime
            .runtime_by_runtime_id
            .lock()
            .await
            .values()
            .cloned()
            .collect()
    };

    runtimes.iter().filter(|runtime| runtime.with_runtime(
        |runtime|
            runtime.presentation() == AppPresentation::Taskbar
            && runtime.host_window_label() == host_window_label
    )).cloned().collect()
}

fn find_runtime_by_app_id_optional_immediate_once(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<Option<SharedRuntime>, String> {
    let runtime_id = {
        let by_app_id = apps_state
            .runtime
            .runtime_id_by_app_id
            .try_lock()
            .map_err(|_| "runtime_id_by_app_id is busy".to_string())?;

        by_app_id.get(app_id).cloned()
    };

    let Some(runtime_id) = runtime_id else {
        return Ok(None);
    };

    let by_runtime_id = apps_state
        .runtime
        .runtime_by_runtime_id
        .try_lock()
        .map_err(|_| "runtime_by_runtime_id is busy".to_string())?;

    Ok(by_runtime_id.get(&runtime_id).cloned())
}

fn find_impostor_runtime_by_victim_app_id_optional_immediate_once(
    apps_state: &State<'_, AppsHostState>,
    victim_app_id: &str,
) -> Result<Option<SharedImpostorRuntime>, String> {
    let runtime_id = {
        let by_victim_app_id = apps_state
            .runtime
            .impostor_runtime_id_by_victim_app_id
            .try_lock()
            .map_err(|_| {
                eprintln!("impostor_runtime_id_by_victim_app_id is busy");
                "impostor_runtime_id_by_victim_app_id is busy".to_string()
            })?;

        by_victim_app_id.get(victim_app_id).cloned()
    };

    let Some(runtime_id) = runtime_id else {
        return Ok(None);
    };

    let by_runtime_id = apps_state
        .runtime
        .impostor_by_runtime_id
        .try_lock()
        .map_err(|_| {
            eprintln!("impostor_by_runtime_id is busy");
            "impostor_by_runtime_id is busy".to_string()
        })?;

    Ok(by_runtime_id.get(&runtime_id).cloned())
}

fn is_busy_error(err: &str) -> bool {
    err.contains(" is busy")
}

fn retry_immediate_lookup<T>(
    label: &str,
    mut lookup: impl FnMut() -> Result<T, String>,
) -> Result<T, String> {
    let deadline = Instant::now() + Duration::from_millis(IMMEDIATE_LOCK_RETRY_TIMEOUT_MS);
    let mut attempts = 0usize;
    let mut last_err: Option<String> = None;

    loop {
        attempts += 1;

        match lookup() {
            Ok(value) => return Ok(value),
            Err(err) if is_busy_error(&err) && Instant::now() < deadline => {
                last_err = Some(err);
                std::thread::yield_now();
            }
            Err(err) => return Err(err),
        }

        if Instant::now() >= deadline {
            let err = format!(
                "{label} is busy after {attempts} attempts over {:?}: {}",
                Duration::from_millis(IMMEDIATE_LOCK_RETRY_TIMEOUT_MS),
                last_err.unwrap_or_else(|| "unknown busy lock".to_string()),
            );
            eprintln!("{err}");
            return Err(err);
        }
    }
}
