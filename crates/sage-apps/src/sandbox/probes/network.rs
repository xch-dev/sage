use tauri::{AppHandle, State};

use crate::AppsHostState;
use crate::sandbox::{BUILTIN_NETWORK_ALLOW_A_ID, BUILTIN_NETWORK_ALLOW_B_ID};

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::poll::poll_network;

pub(in crate::sandbox) async fn run_network_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let run_id = unique_run_id("sandbox-network");
    let app_ids = [BUILTIN_NETWORK_ALLOW_A_ID, BUILTIN_NETWORK_ALLOW_B_ID];

    stop_test_apps(app, apps_state, &app_ids).await;

    start_test_app(
        app,
        apps_state,
        BUILTIN_NETWORK_ALLOW_A_ID,
        &[("runId", run_id.clone())],
        None,
    )
    .await?;
    start_test_app(
        app,
        apps_state,
        BUILTIN_NETWORK_ALLOW_B_ID,
        &[("runId", run_id.clone())],
        None,
    )
    .await?;

    let results = poll_network(apps_state, &run_id, 2, 12_000).await?;
    stop_test_apps(app, apps_state, &app_ids).await;

    for result in &results {
        if result.data.error.is_some() {
            return Ok((false, result.data.error.clone()));
        }

        if !result.data.allowed_ok {
            return Ok((
                false,
                Some(format!(
                    "{} could not reach allowed URL {}.",
                    result.app_id, result.data.allowed_url
                )),
            ));
        }

        if result.data.blocked_ok {
            return Ok((
                false,
                Some(format!(
                    "{} was able to reach blocked URL {}.",
                    result.app_id, result.data.blocked_url
                )),
            ));
        }
    }

    Ok((
        true,
        Some("Network allowlist probes succeeded for allowed URLs and failed for blocked URLs in both flipped configurations.".into()),
    ))
}
