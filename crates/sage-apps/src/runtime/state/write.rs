use tauri::{AppHandle, State};

use crate::AppsHostState;
use crate::runtime::emit_runtime_manager_runtimes_changed;
use crate::runtime::state::types::SageAppRuntimeRecord;
use crate::types::SageApp;

pub async fn write_runtime_and_emit_changed(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    record: SageAppRuntimeRecord,
) {
    write_runtime(apps_state, record).await;
    emit_runtime_manager_runtimes_changed(app, apps_state).await;
}

pub async fn write_runtime_id_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app: &SageApp,
    runtime_id: String,
) {
    let mut runtime_by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
    runtime_by_app_id.insert(app.id().to_string(), runtime_id);
}

pub async fn write_pending_stop_ready(
    apps_state: &State<'_, AppsHostState>,
    request_id: &str,
    tx: tokio::sync::oneshot::Sender<()>,
) {
    let mut pending = apps_state.runtime.pending_stop_ready.lock().await;
    pending.insert(request_id.to_string(), tx);
}

async fn write_runtime(apps_state: &State<'_, AppsHostState>, record: SageAppRuntimeRecord) {
    let mut by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
    by_runtime_id.insert(record.runtime_id().to_string(), record);
}
