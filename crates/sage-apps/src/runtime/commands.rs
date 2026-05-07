use std::collections::BTreeMap;

use serde::{Deserialize};
use specta::Type;
use tauri::{AppHandle, State};

use crate::runtime::start::{create_runtime, CreateRuntimeArgs};
use crate::runtime::state::list_runtimes;
use crate::runtime::stop::SystemKillRuntimeResult;
use crate::runtime::{clear_active_taskbar_runtime, focus_taskbar_runtime, kill_taskbar_runtime, start_app_install_runtime, start_app_update_runtime, start_donation_runtime, start_sandbox_tests_runtime, RuntimeTargetParams, SageAppRuntimeMode, SageAppRuntimeRecordView};
use crate::AppsHostState;
use crate::runtime::events::emit_runtime_manager_runtimes_changed;
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::runtime::workspace::{enter_apps_workspace, leave_apps_workspace};
use crate::types::AppPresentation;

#[derive(Debug, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StartSystemAppArgs {
    AppInstall(StartAppInstallArgs),
    AppUpdate(StartAppUpdateArgs),
    Donation(StartDonationArgs),
    SandboxTests,
}

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartAppInstallArgs {
    pub source: StartAppInstallSource,
}

#[derive(Debug, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StartAppInstallSource {
    SelectSource,
    Url { app_url: String },
}

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartAppUpdateArgs {
    pub mode: StartAppUpdateMode,
    pub app_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StartAppUpdateMode {
    ReviewUpdate,
    ReviewPermissions,
}

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartDonationArgs {
    pub app_id: String,
}

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstalledRuntimeArgs {
    pub app_id: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowTargetParams {
    pub window_label: String,
}

impl StartAppUpdateMode {
    fn query_value(self) -> &'static str {
        match self {
            Self::ReviewUpdate => "review-update",
            Self::ReviewPermissions => "review-permissions",
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn apps_enter_workspace(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
) -> Result<(), String> {
    enter_apps_workspace(&app_handle, &apps_state).await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn apps_leave_workspace(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
) -> Result<(), String> {
    leave_apps_workspace(&app_handle, &apps_state).await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn apps_start_system_app(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: StartSystemAppArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    let runtime = match args {
        StartSystemAppArgs::AppInstall(args) => {
            let mut query = BTreeMap::new();

            match args.source {
                StartAppInstallSource::SelectSource => {
                    query.insert("mode".to_string(), "select-source".to_string());
                }
                StartAppInstallSource::Url { app_url } => {
                    query.insert("mode".to_string(), "url".to_string());
                    query.insert("appUrl".to_string(), app_url);
                }
            }

            start_app_install_runtime(&app, &apps_state, query).await?
        }
        StartSystemAppArgs::AppUpdate(args) => {
            let mut query = BTreeMap::new();

            query.insert("appId".to_string(), args.app_id.clone());
            query.insert("mode".to_string(), args.mode.query_value().to_string());

            start_app_update_runtime(&app, &apps_state, args.app_id, query).await?
        }
        StartSystemAppArgs::Donation(args) => {
            let mut query = BTreeMap::new();

            query.insert("appId".to_string(), args.app_id.clone());

            start_donation_runtime(&app, &apps_state, args.app_id, query).await?
        }
        StartSystemAppArgs::SandboxTests => {
            start_sandbox_tests_runtime(&app, &apps_state).await?
        }
    };

    Ok(runtime.into())
}

#[tauri::command]
#[specta::specta]
pub async fn apps_create_inline_runtime(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: CreateInstalledRuntimeArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    let created_runtime = create_runtime(&app_handle, &apps_state, CreateRuntimeArgs {
        app_id: args.app_id.clone(),
        presentation: AppPresentation::Taskbar,
        mode: SageAppRuntimeMode::Inline,
        debug_layout: false,
        query: BTreeMap::new(),
    }).await.map(Into::into);

    emit_runtime_manager_runtimes_changed(&app_handle, &apps_state).await;

    focus_taskbar_runtime(&app_handle, &apps_state, &args.app_id).await?;

    created_runtime
}

#[tauri::command]
#[specta::specta]
pub async fn apps_list_runtimes(
    apps_state: State<'_, AppsHostState>,
) -> Result<Vec<SageAppRuntimeRecordView>, String> {
    list_runtimes(&apps_state)
        .await
        .map(|runtimes| runtimes.into_iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn apps_focus_taskbar_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    focus_taskbar_runtime(&app, &apps_state, &params.app_id)
        .await
        .map(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_clear_active_taskbar_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: WindowTargetParams,
) -> Result<(), String> {
    clear_active_taskbar_runtime(&app, &apps_state, &params.window_label).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_kill_taskbar_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SystemKillRuntimeResult, String> {
    kill_taskbar_runtime(&app, &apps_state, &params.app_id, "user_kill")
        .await
        .map_err(|_| "Runtime not found".to_string())?;

    Ok(SystemKillRuntimeResult {
        ok: true,
        app_id: params.app_id,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn apps_dev_reload_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    let runtime = crate::runtime::state::get_runtime_by_app_id(&apps_state, &params.app_id)
        .await
        .map_err(|_| "Runtime not found".to_string())?;

    let webview_label = runtime.with_runtime(|runtime| runtime.webview_label().to_string());

    let webview = get_webview_in_sage_window(&app, &webview_label)?;

    webview
        .eval(
            r"
            (() => {
              const url = new URL(window.location.href);
              url.searchParams.set('__sage_dev_reload', String(Date.now()));
              window.location.replace(url.toString());
            })();
            ",
        )
        .map_err(|err| format!("failed to reload runtime webview: {err}"))?;

    Ok(runtime.into())
}
