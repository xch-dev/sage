use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, LogicalPosition, LogicalSize, State};

use crate::AppsHostState;
use crate::runtime::state::{find_runtime_by_runtime_id_optional, list_runtimes};
use crate::runtime::webview_locator::{get_sage_window, get_webview_in_sage_window};
use crate::runtime::{find_active_taskbar_runtime, resolve_running_app, SageAppRuntimeRecord, SageAppRuntimeVisibility, SharedRuntime};
use crate::runtime::events::{emit_active_taskbar_runtime_changed, emit_runtime_manager_runtimes_changed};
use crate::runtime::stop::{kill_runtime_inner};
use crate::runtime::workspace::{ensure_apps_workspace_active};
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

#[derive(Default)]
pub(crate) struct RuntimeChangeSet {
    runtimes_changed: bool,
    active_taskbar_changed: Vec<String>,
}

struct ModalVisibilityCandidate {
    runtime: SharedRuntime,
    eligible: bool,
    priority: i32,
    runtime_id: String,
}

pub(crate) async fn focus_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    ensure_apps_workspace_active(apps_state).await?;

    let resolved_running_app = resolve_running_app(apps_state, app_id)
        .await
        .map_err(|e| format!("failed to resolve running app: {e}"))?;

    assert_taskbar_presentation(&resolved_running_app)?;

    let runtime = resolved_running_app.runtime();
    let runtime_window_identity = runtime_window_identity(&resolved_running_app);

    let mut changes = RuntimeChangeSet::default();

    let current_active_taskbar_runtime =
        find_active_taskbar_runtime(apps_state, &runtime_window_identity.host_window_label).await;

    show_runtime_inner(app_handle, apps_state, &runtime, &mut changes).await?;

    if let Some(current_taskbar_runtime) = current_active_taskbar_runtime {
        let current_taskbar_runtime_id = current_taskbar_runtime.runtime_id();
        let is_same_taskbar_runtime = current_taskbar_runtime_id == runtime_window_identity.runtime_id;
        let current_taskbar_runtime = find_runtime_by_runtime_id_optional(apps_state, &current_taskbar_runtime_id).await;

        if !is_same_taskbar_runtime && let Some(current_active_runtime) = current_taskbar_runtime {
            hide_runtime_inner(app_handle, &current_active_runtime, &mut changes)?;
        }
    }

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &runtime_window_identity.host_window_label,
        &mut changes,
    )
        .await?;

    changes.active_taskbar_changed(&runtime_window_identity.host_window_label);
    changes.emit(app_handle, apps_state).await;

    Ok(runtime)
}

pub(crate) async fn hide_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<SharedRuntime, String> {
    let resolved_running_app = resolve_running_app(apps_state, app_id)
        .await
        .map_err(|e| format!("failed to resolve running app: {e}"))?;

    let runtime = resolved_running_app.runtime();
    let runtime_window_identity = runtime_window_identity(&resolved_running_app);
    let host_window_label = runtime_window_identity.host_window_label;

    let active_taskbar_runtime =
        find_active_taskbar_runtime(apps_state, &host_window_label).await;

    let mut changes = RuntimeChangeSet::default();

    hide_runtime_inner(app_handle, &runtime, &mut changes)?;

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        &host_window_label,
        &mut changes,
    )
        .await?;

    if let Some(active_taskbar_runtime) = active_taskbar_runtime && active_taskbar_runtime.app_id() == app_id {
        changes.active_taskbar_changed(&host_window_label);
    }

    changes.emit(app_handle, apps_state).await;

    Ok(runtime)
}

pub(crate) async fn hide_all_runtimes(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(), String> {
    let mut changes = RuntimeChangeSet::default();

    hide_all_runtimes_inner(
        app_handle,
        apps_state,
        &mut changes,
    )
        .await?;

    changes.emit(app_handle, apps_state).await;

    Ok(())
}

pub(crate) async fn hide_all_runtimes_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    changes: &mut RuntimeChangeSet,
) -> Result<(), String> {
    let sage_window = get_sage_window(app_handle)?;

    if find_active_taskbar_runtime(
        apps_state,
        sage_window.label(),
    )
        .await
        .is_some()
    {
        changes.active_taskbar_changed(sage_window.label());
    }

    for runtime in list_runtimes(apps_state).await? {
        hide_runtime_inner(app_handle, &runtime, changes)?;
    }

    Ok(())
}

pub(crate) async fn clear_active_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    window_label: &str,
) -> Result<(), String> {
    let mut changes = RuntimeChangeSet::default();

    if let Some(active_taskbar_runtime) =
        find_active_taskbar_runtime(apps_state, window_label).await
    {
        hide_runtime_inner(app_handle, &active_taskbar_runtime, &mut changes)?;
        changes.active_taskbar_changed(window_label);
    }

    sync_modal_runtime_visibility(
        app_handle,
        apps_state,
        window_label,
        &mut changes,
    )
        .await?;

    changes.emit(app_handle, apps_state).await;

    Ok(())
}

pub(crate) async fn kill_taskbar_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    reason: &str,
) -> Result<(), String> {
    let mut changes = RuntimeChangeSet::default();

    kill_runtime_inner(
        app_handle,
        apps_state,
        app_id,
        reason,
        &mut changes,
    )
        .await
        .map_err(|err| format!("Failed to kill taskbar runtime: {err:?}"))?;

    changes.emit(app_handle, apps_state).await;

    Ok(())
}

pub(super) async fn sync_modal_runtime_visibility(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    host_window_label: &str,
    changes: &mut RuntimeChangeSet,
) -> Result<(), String> {
    ensure_apps_workspace_active(apps_state).await?;

    let active_taskbar_runtime =
        find_active_taskbar_runtime(apps_state, host_window_label).await;

    let active_app_id =
        active_taskbar_runtime.map(|runtime| runtime.app_id());

    let mut candidates = Vec::new();

    for runtime in list_runtimes(apps_state).await? {
        let Some(candidate) = runtime.with_runtime(|record| {
            if record.host_window_label() != host_window_label {
                return None;
            }

            let AppPresentation::Modal(modal) = record.presentation() else {
                return None;
            };

            let eligible = match active_app_id.as_deref() {
                Some(active_app_id) => modal
                    .visible_over_app_ids()
                    .iter()
                    .any(|app_id| app_id == active_app_id),

                None => modal.visible_over_launchpad(),
            };

            Some((
                eligible,
                modal.priority(),
                record.runtime_id(),
            ))
        }) else {
            continue;
        };

        candidates.push(ModalVisibilityCandidate {
            runtime,
            eligible: candidate.0,
            priority: candidate.1,
            runtime_id: candidate.2,
        });
    }

    let winner_runtime_id = candidates
        .iter()
        .filter(|candidate| candidate.eligible)
        .max_by_key(|candidate| candidate.priority)
        .map(|candidate| candidate.runtime_id.clone());

    for candidate in candidates {
        let should_show =
            Some(candidate.runtime_id.clone()) == winner_runtime_id;

        if should_show {
            show_runtime_inner(
                app_handle,
                apps_state,
                &candidate.runtime,
                changes,
            )
                .await?;
        } else {
            hide_runtime_inner(
                app_handle,
                &candidate.runtime,
                changes,
            )?;
        }
    }

    Ok(())
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

async fn show_runtime_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    runtime: &SharedRuntime,
    changes: &mut RuntimeChangeSet,
) -> Result<(), String> {
    ensure_apps_workspace_active(apps_state).await?;

    if runtime.with_runtime(|runtime| {
        runtime.visibility() == SageAppRuntimeVisibility::Visible
    }) {
        return Ok(());
    }

    let app_webview_label = runtime.with_runtime(SageAppRuntimeRecord::webview_label);
    let webview = get_webview_in_sage_window(app_handle, &app_webview_label)?;

    webview.show().map_err(|err| format!("failed to show webview: {err}"))?;
    webview.set_focus().map_err(|err| format!("failed to focus webview: {err}"))?;

    runtime.with_runtime_mut(SageAppRuntimeRecord::mark_visible);
    changes.runtimes_changed();

    Ok(())
}

fn hide_runtime_inner(
    app_handle: &AppHandle,
    runtime: &SharedRuntime,
    changes: &mut RuntimeChangeSet,
) -> Result<(), String> {
    if runtime.with_runtime(|runtime| {
        runtime.visibility() == SageAppRuntimeVisibility::Hidden
    }) {
        return Ok(());
    }

    let app_webview_label = runtime.with_runtime(SageAppRuntimeRecord::webview_label);
    let webview = get_webview_in_sage_window(app_handle, &app_webview_label)?;

    webview.hide().map_err(|err| format!("failed to hide webview: {err}"))?;
    webview
        .set_position(LogicalPosition::new(0.0, 0.0))
        .map_err(|err| format!("failed to set webview position: {err}"))?;
    webview
        .set_size(LogicalSize::new(1.0, 1.0))
        .map_err(|err| format!("failed to set webview size: {err}"))?;

    runtime.with_runtime_mut(SageAppRuntimeRecord::mark_hidden);
    changes.runtimes_changed();

    Ok(())
}

impl RuntimeChangeSet {
    pub(crate) fn runtimes_changed(&mut self) {
        self.runtimes_changed = true;
    }

    pub(crate) fn active_taskbar_changed(&mut self, window_label: impl Into<String>) {
        let window_label = window_label.into();

        if !self.active_taskbar_changed.contains(&window_label) {
            self.active_taskbar_changed.push(window_label);
        }
    }

    pub(crate) async fn emit(
        self,
        app_handle: &AppHandle,
        apps_state: &State<'_, AppsHostState>,
    ) {
        if self.runtimes_changed {
            emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;
        }

        for window_label in self.active_taskbar_changed {
            let active = find_active_taskbar_runtime(apps_state, &window_label).await;

            emit_active_taskbar_runtime_changed(
                app_handle,
                apps_state,
                &window_label,
                active.as_ref(),
            )
                .await;
        }
    }
}
