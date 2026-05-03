use std::collections::BTreeMap;

use serde::Deserialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::runtime::start::{create_runtime, CreateRuntimeArgs};
use crate::runtime::state::list_runtimes;
use crate::runtime::stop::{kill_runtime, SystemKillRuntimeResult};
use crate::runtime::{
    focus_runtime, hide_runtime, RuntimeTargetParams, SageAppRuntimeMode,
    SageAppRuntimeRecordView, SageAppRuntimeVisibility,
};
use crate::system_apps::{SYSTEM_APP_APP_INSTALL_ID, SYSTEM_APP_APP_UPDATE_ID};
use crate::AppsHostState;
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::types::{AppModalPresentation, AppPresentation};

#[derive(Debug, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StartSystemAppArgs {
    AppInstall(StartAppInstallArgs),
    AppUpdate(StartAppUpdateArgs),
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
pub struct CreateInstalledRuntimeArgs {
    pub app_id: String,
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
pub async fn apps_start_system_app(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: StartSystemAppArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    let create_args = match args {
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

            CreateRuntimeArgs {
                app_id: SYSTEM_APP_APP_INSTALL_ID.to_string(),
                presentation: AppPresentation::Modal(AppModalPresentation {
                    visible_over_app_ids: Vec::new(),
                    visible_over_launchpad: true,
                }),
                mode: SageAppRuntimeMode::Inline,
                visibility: SageAppRuntimeVisibility::Visible,
                debug_layout: false,
                query,
            }
        }
        StartSystemAppArgs::AppUpdate(args) => {
            let mut query = BTreeMap::new();

            query.insert("appId".to_string(), args.app_id.clone());
            query.insert("mode".to_string(), args.mode.query_value().to_string());
            query.insert("visibleOverLaunchpad".to_string(), "true".to_string());

            CreateRuntimeArgs {
                app_id: SYSTEM_APP_APP_UPDATE_ID.to_string(),
                presentation: AppPresentation::Modal(AppModalPresentation {
                    visible_over_app_ids: Vec::from([args.app_id]),
                    visible_over_launchpad: true,
                }),
                mode: SageAppRuntimeMode::Inline,
                visibility: SageAppRuntimeVisibility::Visible,
                debug_layout: false,
                query,
            }
        }
    };

    create_runtime(app, apps_state, create_args).await.map(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_create_inline_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: CreateInstalledRuntimeArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    create_runtime(app, apps_state, CreateRuntimeArgs {
        app_id: args.app_id.clone(),
        presentation: AppPresentation::Taskbar,
        mode: SageAppRuntimeMode::Inline,
        visibility: SageAppRuntimeVisibility::Visible,
        debug_layout: false,
        query: BTreeMap::new(),
    }).await.map(Into::into)
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
pub async fn apps_focus_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    focus_runtime(&app, &apps_state, &params.app_id)
        .await
        .map(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_hide_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SageAppRuntimeRecordView, String> {
    hide_runtime(&app, &apps_state, &params.app_id)
        .await
        .map(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_kill_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    params: RuntimeTargetParams,
) -> Result<SystemKillRuntimeResult, String> {
    kill_runtime(&app, &apps_state, &params.app_id, "user_kill")
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
            r#"
            (() => {
              const url = new URL(window.location.href);
              url.searchParams.set('__sage_dev_reload', String(Date.now()));
              window.location.replace(url.toString());
            })();
            "#,
        )
        .map_err(|err| format!("failed to reload runtime webview: {err}"))?;

    Ok(runtime.into())
}
