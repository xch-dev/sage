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
    let runtime = {
        let mut by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.remove(runtime_id)
    };

    let Some(runtime) = runtime else {
        return;
    };

    let (host_window_label, removed_runtime_id, app_id) = runtime.with_runtime(|runtime| {
        (
            runtime.host_window_label().to_string(),
            runtime.runtime_id(),
            runtime.app_id(),
        )
    });

    remove_active_taskbar_runtime_if_matches(apps_state, &host_window_label, &removed_runtime_id).await;
    remove_runtime_id_by_app_id(apps_state, &app_id).await;
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

pub(in crate::runtime) async fn remove_impostor_runtime_by_victim_app_id(
    apps_state: &State<'_, AppsHostState>,
    victim_app_id: &str,
) {
    let runtime_id = {
        let mut by_victim_app_id = apps_state
            .runtime
            .impostor_runtime_id_by_victim_app_id
            .lock()
            .await;

        by_victim_app_id.remove(victim_app_id)
    };

    if let Some(runtime_id) = runtime_id {
        remove_impostor_runtime_by_runtime_id(apps_state, &runtime_id).await;
    }
}

pub(in crate::runtime) async fn remove_impostor_runtime_by_runtime_id(
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) {
    let runtime = {
        let mut by_runtime_id = apps_state.runtime.impostor_by_runtime_id.lock().await;
        by_runtime_id.remove(runtime_id)
    };

    let Some(runtime) = runtime else {
        return;
    };

    let victim_app_id = runtime.victim_app_id();

    {
        let mut by_victim_app_id = apps_state
            .runtime
            .impostor_runtime_id_by_victim_app_id
            .lock()
            .await;

        if by_victim_app_id.get(&victim_app_id) == Some(&runtime_id.to_string()) {
            by_victim_app_id.remove(&victim_app_id);
        }
    }
}

pub(in crate::runtime) async fn remove_active_taskbar_runtime(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
) {
    let mut active = apps_state
        .runtime
        .active_taskbar_runtime_id_by_host_window_label
        .lock()
        .await;

    if active.get(host_window_label).is_some() {
        active.remove(host_window_label);
    }
}

pub(in crate::runtime) async fn remove_active_taskbar_runtime_if_matches(
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    runtime_id: &str,
) {
    let mut active = apps_state
        .runtime
        .active_taskbar_runtime_id_by_host_window_label
        .lock()
        .await;

    if active
        .get(host_window_label)
        .is_some_and(|active_runtime_id| active_runtime_id == runtime_id)
    {
        active.remove(host_window_label);
    }
}
