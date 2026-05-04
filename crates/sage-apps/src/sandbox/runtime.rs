use crate::AppsHostState;
use crate::runtime::{SageAppRuntimeMode, SageAppRuntimeVisibility};
use crate::runtime::start::{create_runtime, CreateRuntimeArgs};
use crate::runtime::stop::close_runtime_internal;
use std::collections::{BTreeMap, HashMap};
use tauri::{AppHandle, State};
use uuid::Uuid;
use crate::types::AppPresentation;

pub(crate) async fn stop_test_apps(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_ids: &[&str],
) {
    for app_id in app_ids {
        let _ = close_runtime_internal(app, apps_state, app_id).await;
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

    start_internal_runtime_for_sandbox(
        app,
        apps_state,
        app_id,
        SageAppRuntimeVisibility::Hidden,
        query_map.into_iter().collect(),
    )
    .await
}

pub(super) fn unique_run_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}

async fn start_internal_runtime_for_sandbox(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    visibility: SageAppRuntimeVisibility,
    query: BTreeMap<String, String>,
) -> Result<(), String> {
    let debug_test_apps = debug_test_apps_enabled();

    let args = CreateRuntimeArgs {
        app_id: app_id.to_string(),
        presentation: AppPresentation::Taskbar,
        mode: SageAppRuntimeMode::Inline,
        visibility: if debug_test_apps { SageAppRuntimeVisibility::Visible } else { visibility },
        debug_layout: debug_test_apps,
        query,
    };

    create_runtime(app, apps_state, args)
        .await
        .map(|_| ())
}

fn debug_test_apps_enabled() -> bool {
    cfg!(debug_assertions)
        && std::env::var("SAGE_DEBUG_TEST_APPS")
            .map(|v| v == "1")
            .unwrap_or(false)
}
