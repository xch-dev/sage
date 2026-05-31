use tauri::State;

use super::types::{SageAppRuntimeRecord, SharedRuntime};
use crate::AppsHostState;

pub(in crate::runtime) async fn write_runtime(
    apps_state: &State<'_, AppsHostState>,
    runtime: SageAppRuntimeRecord,
) -> SharedRuntime {
    let runtime_id = runtime.runtime_id().to_string();
    let app_id = runtime.app().id().to_string();

    let runtime = SharedRuntime::new(runtime);

    {
        let mut by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
        by_app_id.insert(app_id, runtime_id.clone());
    }

    {
        let mut by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.insert(runtime_id, runtime.clone());
    }

    runtime
}

pub(in crate::runtime) async fn write_pending_stop_ready(
    apps_state: &State<'_, AppsHostState>,
    request_id: &str,
    tx: tokio::sync::oneshot::Sender<()>,
) {
    let mut pending = apps_state.runtime.pending_stop_ready.lock().await;
    pending.insert(request_id.to_string(), tx);
}

pub(in crate::runtime) async fn activate_apps_workspace(apps_state: &State<'_, AppsHostState>) {
    let mut active = apps_state.runtime.apps_workspace_active.write().await;
    *active = true;
}

pub(in crate::runtime) async fn deactivate_apps_workspace(apps_state: &State<'_, AppsHostState>) {
    let mut active = apps_state.runtime.apps_workspace_active.write().await;
    *active = false;
}
