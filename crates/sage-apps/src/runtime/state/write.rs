use tauri::State;

use super::types::{SageAppRuntimeRecord, SharedRuntime};
use crate::AppsHostState;

pub(crate) async fn write_runtime(
    apps_state: &State<'_, AppsHostState>,
    mut runtime: SageAppRuntimeRecord,
) -> SharedRuntime {
    let runtime_id = runtime.runtime_id();
    let app_id = runtime.app().id().clone();
    let runtime = {
        let mut by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;

        if runtime.presentation() == crate::AppPresentation::Taskbar && !runtime.internal() {
            let host_window_label = runtime.host_window_label();
            let next_order = by_runtime_id
                .values()
                .filter_map(|runtime| {
                    runtime.with_runtime(|runtime| {
                        (runtime.presentation() == crate::AppPresentation::Taskbar
                            && !runtime.internal()
                            && runtime.host_window_label() == host_window_label)
                            .then_some(runtime.taskbar_order())
                    })
                })
                .max()
                .map_or(0, |order| order.saturating_add(1));
            runtime.set_taskbar_order(next_order);
        }

        let runtime = SharedRuntime::new(runtime);
        by_runtime_id.insert(runtime_id.clone(), runtime.clone());
        runtime
    };

    let mut by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
    by_app_id.insert(app_id, runtime_id);

    runtime
}

pub(crate) async fn write_pending_stop_ready(
    apps_state: &State<'_, AppsHostState>,
    request_id: &str,
    tx: tokio::sync::oneshot::Sender<()>,
) {
    let mut pending = apps_state.runtime.pending_stop_ready.lock().await;
    pending.insert(request_id.to_string(), tx);
}

pub(crate) async fn activate_apps_workspace(apps_state: &State<'_, AppsHostState>) {
    let mut active = apps_state.runtime.apps_workspace_active.write().await;
    *active = true;
}

pub(crate) async fn deactivate_apps_workspace(apps_state: &State<'_, AppsHostState>) {
    let mut active = apps_state.runtime.apps_workspace_active.write().await;
    *active = false;
}
