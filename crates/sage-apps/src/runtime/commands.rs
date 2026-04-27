use crate::AppsHostState;
use crate::runtime::start::{CreateInlineRuntimeArgs, create_inline_runtime};
use crate::runtime::state::read::list_runtimes;
use crate::runtime::state::types::SageAppRuntimeRecord;
use crate::runtime::stop::{SystemKillRuntimeResult, kill_runtime};
use crate::runtime::{RuntimeTargetParams, focus_runtime, hide_runtime};
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub async fn apps_create_inline_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: CreateInlineRuntimeArgs,
) -> Result<SageAppRuntimeRecord, String> {
    create_inline_runtime(app, apps_state, args).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_list_runtimes(
    apps_state: State<'_, AppsHostState>,
) -> Result<Vec<SageAppRuntimeRecord>, String> {
    list_runtimes(&apps_state).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_focus_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecord, String> {
    focus_runtime(&app, &apps_state, &params.app_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_hide_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecord, String> {
    hide_runtime(&app, &apps_state, &params.app_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_kill_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SystemKillRuntimeResult, String> {
    kill_runtime(&app, &apps_state, &params.app_id, "user_kill").await
}
