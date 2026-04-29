use crate::AppsHostState;
use crate::runtime::state::types::{SharedRuntime};
use std::cmp::Reverse;
use tauri::State;

pub async fn find_runtime_by_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Option<SharedRuntime> {
    let runtime_id = find_runtime_id_by_app_id_optional(apps_state, app_id).await?;
    find_runtime_by_runtime_id_optional(apps_state, &runtime_id).await
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

pub(crate) async fn get_runtime_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    find_runtime_by_app_id_optional(apps_state, app_id)
        .await
        .ok_or_else(|| format!("runtime record not found for app id: {app_id}"))
}

pub(crate) async fn list_runtimes(
    apps_state: &State<'_, AppsHostState>,
) -> Result<Vec<SharedRuntime>, String> {
    let mut runtimes = {
        let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.values().cloned().collect::<Vec<_>>()
    };

    runtimes.retain(|runtime| {
        !runtime.with_runtime(|runtime| runtime.internal())
    });

    runtimes.sort_by_key(|runtime| {
        Reverse(runtime.with_runtime(|runtime| runtime.started_at()))
    });

    Ok(runtimes)
}