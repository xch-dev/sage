use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, LogicalPosition, LogicalSize, State};

use crate::AppsHostState;
use crate::bridge::emit_system_runtime_event_to_listeners;
use crate::bridge::methods::system::{RuntimeManagerRuntimesChangedEvent, RuntimeManagerActiveTaskbarRuntimeChangedEvent};
use crate::runtime::state::{find_runtime_by_runtime_id_optional, list_runtimes};
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::runtime::{find_active_taskbar_runtime, resolve_running_app, SageAppRuntimeRecord, SageAppRuntimeRecordView, SageAppRuntimeVisibility, SharedRuntime};
use crate::runtime::stop::kill_runtime;
use crate::types::{AppPresentation, ResolvedRunningApp};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTargetParams {
    pub app_id: String,
}

struct RuntimeWindowIdentity {
    runtime_id: String,
    host_window_label: String,
}

pub(crate) async fn emit_runtime_manager_runtimes_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) {
    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    let runtime_records = runtimes
        .iter()
        .map(Into::into)
        .collect::<Vec<SageAppRuntimeRecordView>>();

    let event = RuntimeManagerRuntimesChangedEvent::new(runtime_records);

    emit_system_runtime_event_to_listeners(app_handle, apps_state, event).await;
}

pub(crate) async fn focus_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    let resolved_running_app = resolve_running_app(apps_state, app_id).await
        .map_err(|e| format!("failed to resolve running app: {e}"))?;
    assert_taskbar_presentation(&resolved_running_app)?;

    let runtime_window_identity = runtime_window_identity(&resolved_running_app);
    let current_active_taskbar_runtime = find_active_taskbar_runtime(
        apps_state,
        &runtime_window_identity.host_window_label
    ).await;

    show_runtime(app_handle, &resolved_running_app)?;

    if let Some(current_active_taskbar_runtime) = current_active_taskbar_runtime {
        let current_active_taskbar_runtime_id = current_active_taskbar_runtime.runtime_id();
        if current_active_taskbar_runtime_id != runtime_window_identity.runtime_id {
            hide_runtime_by_runtime_id_if_present(app_handle, apps_state, &current_active_taskbar_runtime_id).await;
        }
    }

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &runtime_window_identity.host_window_label,
    ).await;

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;
    emit_active_taskbar_runtime_changed(
        app_handle,
        apps_state,
        &runtime_window_identity.host_window_label,
        Some(&resolved_running_app.runtime())
    ).await;

    Ok(resolved_running_app.runtime())
}

pub(crate) async fn hide_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    let resolved_running_app = resolve_running_app(apps_state, app_id).await
        .map_err(|e| format!("failed to resolve running app: {e}"))?;
    let runtime = resolved_running_app.runtime();
    if runtime.with_runtime(|runtime| runtime.visibility() == SageAppRuntimeVisibility::Hidden) {
        return Ok(runtime);
    }

    let runtime_window_identity = runtime_window_identity(&resolved_running_app);
    let host_window_label = runtime_window_identity.host_window_label;
    let active_taskbar_runtime = find_active_taskbar_runtime(
        apps_state,
        &host_window_label
    ).await;

    hide_runtime_inner(app_handle, &runtime)?;

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &host_window_label,
    ).await;

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;
    if let Some(active_taskbar_runtime) = active_taskbar_runtime && active_taskbar_runtime.app_id() == app_id {
        emit_active_taskbar_runtime_changed(app_handle, apps_state, &host_window_label, None).await;
    }

    Ok(runtime)
}

pub(crate) async fn clear_active_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    window_label: &str,
) -> Result<(), String> {
    let Some(active_taskbar_runtime) = find_active_taskbar_runtime(apps_state, window_label).await else {
        return Ok(());
    };

    hide_runtime_inner(app_handle, &active_taskbar_runtime)?;

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;
    emit_active_taskbar_runtime_changed(app_handle, apps_state, window_label, None).await;

    Ok(())
}

pub(crate) async fn kill_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    reason: &str,
) -> Result<(), String> {
    let resolved_running_app = resolve_running_app(apps_state, app_id)
        .await.map_err(|err| format!("Failed to resolve running app: {err}"))?;
    let host_window_label = resolved_running_app.runtime().with_runtime(SageAppRuntimeRecord::host_window_label);
    let active_taskbar_runtime = find_active_taskbar_runtime(apps_state, &host_window_label).await;
    kill_runtime(app_handle, apps_state, app_id, reason)
        .await
        .map_err(|err| format!("Failed to kill taskbar runtime: {err}"))?;

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;
    if let Some(active_taskbar_runtime) = active_taskbar_runtime && active_taskbar_runtime.app_id() == app_id {
        emit_active_taskbar_runtime_changed(app_handle, apps_state, &host_window_label, None).await;
    }

    Ok(())
}

async fn sync_modal_runtime_visibility(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
) {
    let active_taskbar_runtime = find_active_taskbar_runtime(apps_state, host_window_label).await;
    let active_app_id = active_taskbar_runtime.map(|active_taskbar_runtime| active_taskbar_runtime.app_id());

    let Ok(runtimes) = list_runtimes(apps_state).await else {
        return;
    };

    for runtime in runtimes {
        let Some(decision) = runtime.with_runtime(|record| {
            if record.host_window_label() != host_window_label {
                return None;
            }

            let AppPresentation::Modal(modal) = record.presentation() else {
                return None;
            };

            let should_be_visible = match active_app_id.as_deref() {
                Some(active_app_id) => modal
                    .visible_over_app_ids
                    .iter()
                    .any(|app_id| app_id == active_app_id),
                None => modal.visible_over_launchpad,
            };

            Some((
                record.webview_label().to_string(),
                should_be_visible,
            ))
        }) else {
            continue;
        };

        let (webview_label, should_be_visible) = decision;

        let Ok(webview) = get_webview_in_sage_window(app_handle, &webview_label) else {
            continue;
        };

        if should_be_visible {
            let _ = webview.show();
            runtime.with_runtime_mut(SageAppRuntimeRecord::mark_visible);
        } else {
            let _ = webview.hide();
            runtime.with_runtime_mut(SageAppRuntimeRecord::mark_hidden);
        }
    }
}

async fn hide_runtime_by_runtime_id_if_present(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    runtime_id: &str,
) {
    let Some(runtime) = find_runtime_by_runtime_id_optional(apps_state, runtime_id).await else {
        return;
    };

    let webview_label = runtime.with_runtime(|record| record.webview_label().to_string());

    if let Ok(webview) = get_webview_in_sage_window(app, &webview_label) {
        let _ = webview.hide();
    }

    runtime.with_runtime_mut(SageAppRuntimeRecord::mark_hidden);
}

async fn emit_active_taskbar_runtime_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    runtime: Option<&SharedRuntime>,
) {
    let (runtime_id, app_id) = match runtime {
        Some(shared_runtime) => shared_runtime.with_runtime(
            |record| (
                Some(record.runtime_id()),
                Some(record.app_id().clone())
            )
        ),
        None => (None, None),
    };
    let () = emit_system_runtime_event_to_listeners(app_handle, apps_state, RuntimeManagerActiveTaskbarRuntimeChangedEvent {
        host_window_label: host_window_label.to_string(),
        app_id,
        runtime_id,
    }).await;
}

fn runtime_window_identity(resolved_running_app: &ResolvedRunningApp) -> RuntimeWindowIdentity {
    resolved_running_app.runtime().with_runtime(|record| RuntimeWindowIdentity {
        runtime_id: record.runtime_id(),
        host_window_label: record.host_window_label().to_string(),
    })
}

fn assert_taskbar_presentation(resolved_running_app: &ResolvedRunningApp) -> Result<(), String> {
    if !resolved_running_app.runtime().is_taskbar() {
        return Err("Cannot focus non-taskbar runtime".to_string());
    }

    Ok(())
}

fn show_runtime(app_handle: &AppHandle, resolved_running_app: &ResolvedRunningApp) -> Result<(), String> {
    let app_webview_label = resolved_running_app
        .runtime()
        .with_runtime(SageAppRuntimeRecord::webview_label);
    let webview = get_webview_in_sage_window(app_handle, &app_webview_label)?;
    webview
        .show()
        .map_err(|err| format!("failed to show webview: {err}"))?;
    webview
        .set_focus()
        .map_err(|err| format!("failed to focus webview: {err}"))?;

    let runtime = resolved_running_app.runtime();
    runtime.with_runtime_mut(SageAppRuntimeRecord::mark_visible);

    Ok(())
}

fn hide_runtime_inner(app_handle: &AppHandle, runtime: &SharedRuntime) -> Result<(), String> {
    let app_webview_label = runtime
        .with_runtime(SageAppRuntimeRecord::webview_label);
    let webview = get_webview_in_sage_window(app_handle, &app_webview_label)?;
    webview
        .hide()
        .map_err(|err| format!("failed to hide webview: {err}"))?;
    webview.set_position(LogicalPosition::new(0.0, 0.0))
        .map_err(|err| format!("failed to set webview position: {err}"))?;
    webview.set_size(LogicalSize::new(1.0, 1.0))
        .map_err(|err| format!("failed to set webview size: {err}"))?;

    runtime.with_runtime_mut(SageAppRuntimeRecord::mark_hidden);

    Ok(())
}
