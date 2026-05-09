use crate::AppsHostState;
use crate::runtime::start::{start_sandbox_test};
use crate::runtime::stop::close_runtime_internal;
use std::collections::{BTreeMap, HashMap};
use tauri::{AppHandle, State};
use uuid::Uuid;

pub(crate) async fn stop_test_apps(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_ids: &[&str],
) {
    for app_id in app_ids {
        close_runtime_internal(app, apps_state, app_id).await;
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
