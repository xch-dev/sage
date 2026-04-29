use crate::AppsHostState;
use crate::runtime::start::{CreateRuntimeArgs, create_runtime};
use crate::runtime::state::list_runtimes;
use crate::runtime::stop::{SystemKillRuntimeResult, kill_runtime};
use crate::runtime::{RuntimeTargetParams, focus_runtime, hide_runtime, SageAppRuntimeRecordView};
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub async fn apps_create_inline_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    create_runtime(app, apps_state, args).await.into()
}

#[tauri::command]
#[specta::specta]
pub async fn apps_list_runtimes(
    apps_state: State<'_, AppsHostState>,
) -> Result<Vec<SageAppRuntimeRecordView>, String> {
    list_runtimes(&apps_state).await.map(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_focus_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    focus_runtime(&app, &apps_state, &params.app_id).await.into()
}

#[tauri::command]
#[specta::specta]
pub async fn apps_hide_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    hide_runtime(&app, &apps_state, &params.app_id).await.into()
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
