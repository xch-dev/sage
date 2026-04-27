use crate::AppsHostState;
use crate::runtime::state::types::SageAppRuntimeRecord;
use tauri::State;

pub async fn find_runtime_id_by_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Option<String> {
    let runtime_by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
    runtime_by_app_id.get(app_id).cloned()
}

pub async fn get_runtime_id_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<String, String> {
    find_runtime_id_by_app_id_optional(apps_state, app_id)
        .await
        .ok_or_else(|| format!("runtime not found for app id: {app_id}"))
}

pub async fn find_runtime_by_runtime_id_optional(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) -> Option<SageAppRuntimeRecord> {
    let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
    by_runtime_id.get(runtime_id).cloned()
}

pub async fn get_runtime_by_runtime_id(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) -> Result<SageAppRuntimeRecord, String> {
    find_runtime_by_runtime_id_optional(apps_state, runtime_id)
        .await
        .ok_or_else(|| format!("runtime record not found for runtime id: {runtime_id}"))
}

pub async fn find_runtime_by_app_id_optional(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Option<SageAppRuntimeRecord> {
    let Some(runtime_id) = find_runtime_id_by_app_id_optional(apps_state, app_id).await else {
        return None;
    };
    find_runtime_by_runtime_id_optional(apps_state, &runtime_id).await
}

pub async fn get_runtime_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SageAppRuntimeRecord, String> {
    find_runtime_by_app_id_optional(apps_state, app_id)
        .await
        .ok_or_else(|| format!("runtime record not found for app id: {app_id}"))
}

pub async fn list_runtimes(
    apps_state: &State<'_, AppsHostState>,
) -> Result<Vec<SageAppRuntimeRecord>, String> {
    let mut records = {
        let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.values().cloned().collect::<Vec<_>>()
    };

    records.retain(|record| !record.internal);
    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    Ok(records)
}
