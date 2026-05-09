use std::time::Duration;
use futures::future::join_all;

use tauri::{AppHandle, Manager, State};
use crate::{AppsHostState};
use crate::host::AppState;
use crate::lifecycle::{apps_root, list_installed_apps_internal};
use crate::lifecycle::update::logic::check_app_update_inner;
use crate::types::ListedSageApp;

pub fn start_background_app_update_checker(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
        interval.tick().await;

        loop {
            interval.tick().await;

            if let Err(err) = run_background_app_update_check(&app_handle).await {
                tracing::error!(
                    error = %err,
                    "background app update check failed"
                );
            }
        }
    });
}

async fn run_background_app_update_check(app_handle: &AppHandle) -> anyhow::Result<()> {
    let app_state: State<'_, AppState> = app_handle.state();
    let host_state: State<'_, AppsHostState> = app_handle.state();

    let base_path = {
        let state = app_state.lock().await;
        state.path.clone()
    };

    let installed_apps = list_installed_apps_internal(&apps_root(&base_path))?;

    let app_ids = installed_apps
        .into_iter()
        .filter_map(|installed_app| match installed_app {
            ListedSageApp::User(app) => Some(app.common().id().to_string()),
            ListedSageApp::System(_) | ListedSageApp::Corrupted(_) => None,
        })
        .collect::<Vec<_>>();

    let checks = app_ids.into_iter().map(|app_id| {
        let app_handle = app_handle.clone();
        let host_state = host_state.clone();

        async move {
            let result = check_app_update_inner(&app_handle, &host_state, &app_id).await;

            if let Err(err) = &result {
                tracing::warn!(
                error = %err,
                app_id = %app_id,
                "failed to check app update in background"
            );
            }

            result
        }
    });

    let _results = join_all(checks).await;

    Ok(())
}
