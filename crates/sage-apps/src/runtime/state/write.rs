use tauri::State;
use crate::AppsHostState;
use crate::runtime::SharedRuntime;
use crate::runtime::state::types::{SageAppRuntimeRecord};

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
