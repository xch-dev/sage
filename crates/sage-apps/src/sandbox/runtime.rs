use crate::AppsHostState;
use crate::runtime::apps_create_inline_runtime;
use crate::runtime::start::CreateInlineRuntimeArgs;
use crate::runtime::stop::close_runtime_internal;
use crate::security::RUNTIME_APPS_PREFIX;
use std::collections::{BTreeMap, HashMap};
use tauri::{AppHandle, State};
use uuid::Uuid;

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
    path: Option<String>,
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
        false,
        path,
        query_map.into_iter().collect(),
    )
    .await
}

pub(crate) async fn run_clear_cycle_phase_runtime(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    run_id: &str,
    phase_string: String,
) -> Result<(), String> {
    let _ = close_runtime_internal(app, apps_state, app_id).await;

    let mut query = BTreeMap::new();
    query.insert("runId".to_string(), run_id.to_string());
    query.insert("phase".to_string(), phase_string);
    query.insert("appId".to_string(), app_id.to_string());

    start_internal_runtime_for_sandbox(
        app,
        apps_state,
        app_id,
        false,
        Some(RuntimeApp::StorageClearProbe.path()),
        query,
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
    visible: bool,
    path: Option<String>,
    query: BTreeMap<String, String>,
) -> Result<(), String> {
    let debug_test_apps = debug_test_apps_enabled();

    let args = CreateInlineRuntimeArgs {
        app_id: app_id.to_string(),
        visible: if debug_test_apps { true } else { visible },
        internal: true,
        debug_layout: debug_test_apps,
        path,
        query,
    };

    apps_create_inline_runtime(app.clone(), apps_state.clone(), args)
        .await
        .map(|_| ())
}

enum RuntimeApp {
    StorageClearProbe,
}

impl RuntimeApp {
    fn base(&self) -> &'static str {
        match self {
            Self::StorageClearProbe => "storage-clear-probe",
        }
    }

    fn path(&self) -> String {
        format!("{RUNTIME_APPS_PREFIX}{}/{}", self.base(), "index.html")
    }
}

fn debug_test_apps_enabled() -> bool {
    cfg!(debug_assertions)
        && std::env::var("SAGE_DEBUG_TEST_APPS")
            .map(|v| v == "1")
            .unwrap_or(false)
}
