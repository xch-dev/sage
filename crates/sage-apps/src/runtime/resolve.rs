use std::collections::BTreeMap;
use std::fmt::Display;
use crate::lifecycle::read_installed_app_by_id;
use crate::runtime::state::SageAppRuntimeKind;
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::sandbox::build_builtin_test_app;
use crate::types::{ResolvedApp, ResolvedRunningApp, ResolvedStoppedApp, SageApp, SharedSageApp};
use tauri::{AppHandle, Manager, State};
use url::Url;
use crate::AppsHostState;
use crate::runtime::find_runtime_by_app_id_optional;
use crate::runtime::stop::close_runtime_internal;
use tokio::time::{sleep, Duration};

const MAX_STOP_RESOLVE_ATTEMPTS: usize = 5;

#[derive(Debug)]
pub enum ResolveError {
    AppDirMissing,
    NotFound(String),
    BuildFailed(String),
}

#[derive(Debug, Copy, Clone)]
pub enum ResolveStoppedError {
    AppDirMissing,
    CloseAttemptsHit
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ResolveError::AppDirMissing => "app dir missing".to_string(),
            ResolveError::NotFound(msg) => msg.clone(),
            ResolveError::BuildFailed(msg) => msg.clone(),
        };
        write!(f, "{}", str)
    }
}

impl Display for ResolveStoppedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ResolveStoppedError::CloseAttemptsHit => "too many close attempts".to_string(),
            ResolveStoppedError::AppDirMissing => "app dir missing".to_string(),
        };
        write!(f, "{}", str)
    }
}

pub fn app_id_from_webview_label(label: &str) -> Option<&str> {
    if let Some(app_id) = label.strip_prefix("app-") {
        return Some(app_id);
    }

    if let Some(app_id) = label.strip_prefix("system-app-") {
        return Some(app_id);
    }

    None
}

pub fn runtime_kind_for_app(app: &SageApp) -> SageAppRuntimeKind {
    match app {
        SageApp::User(_) => SageAppRuntimeKind::User,
        SageApp::System(_) => SageAppRuntimeKind::System,
    }
}

pub fn protocol_scheme_for_app(app: &SharedSageApp) -> &'static str {
    if app.is_system_app() {
        return "sage-system-app";
    }

    "sage-app"
}

pub fn is_allowed_app_url(url: &Url, app: &SharedSageApp) -> bool {
    url.scheme() == protocol_scheme_for_app(app) && url.host_str() == Some(&app.origin_id())
}

pub fn build_entry_src(
    app: &SharedSageApp,
    query: BTreeMap<String, String>,
) -> Url {
    let scheme = protocol_scheme_for_app(app);

    let entry_file = app.with(|app| app.entry_file());
    let mut url = Url::parse(&format!("{scheme}://{}/{}", app.origin_id(), entry_file))
        .expect("failed to build app entry URL");

    for (key, value) in query {
        url.query_pairs_mut().append_pair(&key, &value);
    }

    url
}

pub async fn resolve_stopped_app(
    app: &AppHandle,
    app_id: &str,
) -> Result<ResolvedStoppedApp, ResolveStoppedError> {
    let apps_state: State<'_, AppsHostState> = app.state();
    let mut delay = Duration::from_millis(25);

    for attempt in 1..=MAX_STOP_RESOLVE_ATTEMPTS {
        let resolved_app = resolve_app(app, app_id).await.map_err(|e| match e {
            ResolveError::AppDirMissing
            | ResolveError::NotFound(_)
            | ResolveError::BuildFailed(_) => ResolveStoppedError::AppDirMissing,
        })?;
        match resolved_app {
            ResolvedApp::Stopped(stopped) => {
                return Ok(stopped);
            }

            ResolvedApp::Running(running) => {
                drop(running);

                close_runtime_internal(app, &apps_state, app_id).await;

                if attempt < MAX_STOP_RESOLVE_ATTEMPTS {
                    sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }

    Err(ResolveStoppedError::CloseAttemptsHit)
}

pub async fn resolve_app(app: &AppHandle, app_id: &str) -> Result<ResolvedApp, ResolveError> {
    let state: State<'_, AppsHostState> = app.state();
    let lock = state.inner().operation_lock_for_app(app_id);

    let guard = lock.lock_owned().await;

    if let Some(runtime) = find_runtime_by_app_id_optional(&state, app_id).await {
        drop(guard);

        return Ok(ResolvedApp::Running(ResolvedRunningApp::new(runtime)));
    }

    let base_path = app
        .path()
        .app_data_dir()
        .map_err(|_| ResolveError::AppDirMissing)?;

    if let Ok(app) = read_installed_app_by_id(&base_path, app_id) {
        return Ok(
            ResolvedApp::Stopped(ResolvedStoppedApp::new(
                SharedSageApp::new(SageApp::User(app)),
                guard
            ))
        );
    }

    let Some(app) = build_builtin_test_app(app_id)
        .map_err(|err| ResolveError::BuildFailed(format!(
            "failed to resolve builtin sandbox app {app_id}: {err}"
        )))?
    else {
        return Err(ResolveError::NotFound(format!("failed to resolve app {app_id}")));
    };

    Ok(ResolvedApp::Stopped(ResolvedStoppedApp::new(
        SharedSageApp::new(app),
        guard,
    )))
}

pub(crate) async fn assert_bridge_origin(
    app_handle: &AppHandle,
    webview_label: &String,
) -> Result<SharedSageApp, String> {
    let app_id = app_id_from_webview_label(webview_label)
        .ok_or_else(|| format!("invalid app runtime label: {webview_label}"))?;

    let runtime = find_runtime_by_app_id_optional(&app_handle.state(), app_id).await
        .ok_or_else(|| format!("failed to find runtime for app {app_id}"))?;
    let app = runtime.app();

    if !app.webview_label_matches(webview_label) {
        return Err(format!(
            "bridge denied for {webview_label}: webview label mismatch"
        ));
    }

    let app_webview = get_webview_in_sage_window(app_handle, webview_label)?;

    let current_url = app_webview
        .url()
        .map_err(|e| format!("failed to read current webview url: {e}"))?;

    if !is_allowed_app_url(&current_url, &app) {
        return Err(format!(
            "bridge denied for {webview_label}: current url {} is outside {}://{}/...",
            current_url,
            protocol_scheme_for_app(&app),
            app.origin_id()
        ));
    }

    Ok(app)
}
