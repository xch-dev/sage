use crate::AppsHostState;
use tauri::State;

pub(in crate::runtime) async fn remove_runtime_id_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) {
    let mut runtime_id_by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
    runtime_id_by_app_id.remove(app_id);
}

pub(in crate::runtime) async fn remove_runtime_by_runtime_id(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) {
    let mut runtime_by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
    runtime_by_runtime_id.remove(runtime_id);
}

pub(in crate::runtime) async fn remove_before_stop_listeners_by_app_id(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) {
    let mut listeners = apps_state
        .runtime
        .before_stop_listeners_by_app_id
        .lock()
        .await;
    listeners.remove(app_id);
}

pub(in crate::runtime) async fn remove_pending_stop_ready(
    apps_state: &State<'_, AppsHostState>,
    request_id: &String,
) {
    let mut pending = apps_state.runtime.pending_stop_ready.lock().await;
    pending.remove(request_id);
}
