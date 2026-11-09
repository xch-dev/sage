use std::time::Duration;

use futures::future::join_all;
use tauri::{AppHandle, Manager, State};

use crate::{
    AppsHostState, ListedSageApp, RecoverableAppUpdateOutcome, apply_recoverable_app_update_inner,
    check_app_update_inner, list_installed_apps_internal, try_auto_apply_pending_update,
};

pub fn start_background_app_update_checker(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        tracing::info!("starting background app update checker");

        let mut interval = tokio::time::interval(Duration::from_mins(10));

        loop {
            if let Err(err) = run_background_app_update_check(&app_handle).await {
                tracing::error!(
                    error = %err,
                    "background app update check failed"
                );
            }

            interval.tick().await;
        }
    });
}

async fn run_background_app_update_check(app_handle: &AppHandle) -> anyhow::Result<()> {
    let host_state: State<'_, AppsHostState> = app_handle.state();

    let installed_apps = list_installed_apps_internal(&host_state.db).await?;

    let app_ids = installed_apps
        .into_iter()
        .filter_map(|installed_app| match installed_app {
            ListedSageApp::User(app) => Some(app.common().id().to_string()),
            ListedSageApp::Corrupted(app) if app.source().is_some() => Some(app.id().to_string()),
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

                return result;
            }

            let auto_update_enabled = host_state
                .db
                .get_auto_update_enabled()
                .await
                .unwrap_or(false);

            if auto_update_enabled {
                let auto_apply = match host_state.db.get_user_app(&app_id).await {
                    Ok(Some(_)) => {
                        try_auto_apply_pending_update(&app_handle, &host_state, &app_id).await
                    }
                    Err(_) => apply_recoverable_app_update_inner(
                        &app_handle,
                        &host_state,
                        &app_id,
                        None,
                        None,
                        false,
                    )
                    .await
                    .map(|outcome| matches!(outcome, RecoverableAppUpdateOutcome::Applied(_))),
                    Ok(None) => Ok(false),
                };

                match auto_apply {
                    Ok(true) => {
                        tracing::info!(
                            app_id = %app_id,
                            "auto-applied app update"
                        );
                    }
                    Ok(false) => {}
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            app_id = %app_id,
                            "failed to auto-apply app update"
                        );
                    }
                }
            }

            result
        }
    });

    let _results = join_all(checks).await;

    Ok(())
}
