use std::collections::{BTreeMap, HashMap};

use tauri::{AppHandle, State};
use tokio::time::{Duration, sleep, timeout};
use uuid::Uuid;

use crate::{
    AppsHostState, close_runtime_internal, find_webview_in_sage_window, start_sandbox_test,
};

const TEST_WEBVIEW_CLOSE_TIMEOUT: Duration = Duration::from_secs(2);
const TEST_WEBVIEW_CLOSE_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub(crate) async fn stop_test_apps(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_ids: &[&str],
) -> Result<(), String> {
    let mut errors = Vec::new();

    for app_id in app_ids {
        let webview_label = format!("app-{app_id}");
        close_runtime_internal(app, apps_state, app_id).await;

        if timeout(TEST_WEBVIEW_CLOSE_TIMEOUT, async {
            while find_webview_in_sage_window(app, &webview_label).is_some() {
                sleep(TEST_WEBVIEW_CLOSE_POLL_INTERVAL).await;
            }
        })
        .await
        .is_err()
        {
            errors.push(format!(
                "timed out waiting for sandbox test webview '{app_id}' to close"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub(crate) async fn start_test_app(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    query: &[(&str, String)],
) -> Result<(), String> {
    let mut query_map = HashMap::new();

    query_map.insert("appId".to_string(), app_id.to_string());

    for (k, v) in query {
        query_map.insert((*k).to_string(), v.clone());
    }

    start_internal_runtime_for_sandbox(app, apps_state, app_id, query_map.into_iter().collect())
        .await
}

pub(super) fn unique_run_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}

async fn start_internal_runtime_for_sandbox(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    query: BTreeMap<String, String>,
) -> Result<(), String> {
    start_sandbox_test(app, apps_state, app_id, query).await
}
