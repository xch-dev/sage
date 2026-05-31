use tauri::{AppHandle, State};

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::poll::poll_isolation;
use crate::{
    AppsHostState, BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID, BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
};

pub(in crate::sandbox) async fn run_isolation_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let run_id = unique_run_id("sandbox-isolation");
    let app_ids = [
        BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
        BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID,
    ];

    stop_test_apps(app, apps_state, &app_ids).await;

    start_test_app(
        app,
        apps_state,
        BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
        &[("runId", run_id.clone())],
    )
    .await?;
    start_test_app(
        app,
        apps_state,
        BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID,
        &[("runId", run_id.clone())],
    )
    .await?;

    let results = poll_isolation(apps_state, &run_id, 2, 2_000).await?;
    stop_test_apps(app, apps_state, &app_ids).await;

    let persistent = results
        .iter()
        .find(|r| r.app_id == BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID);
    let incognito = results
        .iter()
        .find(|r| r.app_id == BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID);

    let Some(persistent) = persistent else {
        return Ok((false, Some("Missing persistent isolation result.".into())));
    };
    let Some(incognito) = incognito else {
        return Ok((false, Some("Missing incognito isolation result.".into())));
    };

    for (label, result) in [
        ("persistent", &persistent.data),
        ("incognito", &incognito.data),
    ] {
        if result.error.is_some() {
            return Ok((
                false,
                Some(format!(
                    "{label} isolation probe reported error: {}",
                    result.error.clone().unwrap_or_default()
                )),
            ));
        }

        if result.local_storage_visible || result.indexed_db_visible {
            return Ok((
                false,
                Some(format!(
                    "{label} probe was able to observe Sage probe data."
                )),
            ));
        }
    }

    Ok((
        true,
        Some("Both sandbox probe modes were unable to observe Sage probe data.".into()),
    ))
}
