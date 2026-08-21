use std::collections::BTreeMap;
use std::fmt::Display;

use tauri::{AppHandle, Manager, State};
use tokio::time::{Duration, sleep};
use url::Url;

use crate::{
    AppsHostState, GetRuntimeError, ResolvedApp, ResolvedRunningApp, ResolvedStoppedApp, SageApp,
    SharedSageApp, build_builtin_system_app, build_builtin_test_app, close_runtime_internal,
    find_runtime_by_app_id_optional,
};

const MAX_STOP_RESOLVE_ATTEMPTS: usize = 5;

#[derive(Debug)]
pub enum ResolveError {
    NotFound(String),
    BuildFailed(String),
}

#[derive(Debug, Copy, Clone)]
pub enum ResolveStoppedError {
    AppDirMissing,
    CloseAttemptsHit,
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ResolveError::NotFound(msg) | ResolveError::BuildFailed(msg) => msg.clone(),
        };
        write!(f, "{str}")
    }
}

impl Display for ResolveStoppedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ResolveStoppedError::CloseAttemptsHit => "too many close attempts".to_string(),
            ResolveStoppedError::AppDirMissing => "app dir missing".to_string(),
        };
        write!(f, "{str}")
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

pub fn protocol_scheme_for_app(app: &SharedSageApp) -> &'static str {
    if app.is_system_app() {
        return "sage-system-app";
    }

    "sage-app"
}

pub fn is_allowed_app_url(url: &Url, app: &SharedSageApp) -> bool {
    let protocol_scheme = protocol_scheme_for_app(app);
    let origin_id = app.origin_id();

    is_allowed_app_origin_url(url, protocol_scheme, &origin_id)
}

fn is_allowed_app_origin_url(url: &Url, protocol_scheme: &str, origin_id: &str) -> bool {
    if url.scheme() == protocol_scheme && url.host_str() == Some(origin_id) {
        return true;
    }

    #[cfg(target_os = "windows")]
    if is_webview2_mapped_app_origin(url, protocol_scheme, origin_id) {
        return true;
    }

    false
}

#[cfg(any(target_os = "windows", test))]
fn is_webview2_mapped_app_origin(url: &Url, protocol_scheme: &str, origin_id: &str) -> bool {
    // WebView2 cannot navigate directly to custom protocols, so Wry maps
    // `{scheme}://{host}` to `https://{scheme}.{host}` on Windows. Navigation
    // callbacks and the current URL expose that mapped URL even though custom
    // protocol requests are converted back before they reach Sage.
    let webview2_host = format!("{protocol_scheme}.{origin_id}");
    url.scheme() == "https" && url.port().is_none() && url.host_str() == Some(&webview2_host)
}

pub fn build_entry_src_for(
    identity_app: &SharedSageApp,
    content_app: &SharedSageApp,
    query: BTreeMap<String, String>,
) -> Url {
    let scheme = protocol_scheme_for_app(identity_app);
    let entry_file = content_app.with(SageApp::entry_file);

    let mut url = Url::parse(&format!(
        "{scheme}://{}/{}",
        identity_app.origin_id(),
        entry_file
    ))
    .expect("failed to build app entry URL");

    for (key, value) in query {
        url.query_pairs_mut().append_pair(&key, &value);
    }

    url
}

pub fn build_entry_src(app: &SharedSageApp, query: BTreeMap<String, String>) -> Url {
    build_entry_src_for(app, app, query)
}

pub(crate) async fn resolve_running_app(
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<ResolvedRunningApp, GetRuntimeError> {
    let runtime = find_runtime_by_app_id_optional(apps_state, app_id)
        .await
        .ok_or(GetRuntimeError::NotFound)?;

    Ok(ResolvedRunningApp::new(runtime))
}

pub(crate) async fn resolve_stopped_app(
    app: &AppHandle,
    app_id: &str,
) -> Result<ResolvedStoppedApp, ResolveStoppedError> {
    let apps_state: State<'_, AppsHostState> = app.state();
    let mut delay = Duration::from_millis(25);

    for attempt in 1..=MAX_STOP_RESOLVE_ATTEMPTS {
        let resolved_app = resolve_app(app, app_id).await.map_err(|e| match e {
            ResolveError::NotFound(_) | ResolveError::BuildFailed(_) => {
                ResolveStoppedError::AppDirMissing
            }
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

pub(crate) async fn resolve_app(
    app: &AppHandle,
    app_id: &str,
) -> Result<ResolvedApp, ResolveError> {
    resolve_app_with_extra(app, app_id, |_| Ok(None)).await
}

pub(crate) async fn resolve_app_with_extra(
    app: &AppHandle,
    app_id: &str,
    extra: impl FnOnce(&str) -> Result<Option<SageApp>, ResolveError>,
) -> Result<ResolvedApp, ResolveError> {
    let state: State<'_, AppsHostState> = app.state();
    let lock = state.inner().operation_lock_for_app(app_id);

    let guard = lock.lock_owned().await;

    if let Some(runtime) = find_runtime_by_app_id_optional(&state, app_id).await {
        drop(guard);
        return Ok(ResolvedApp::Running(ResolvedRunningApp::new(runtime)));
    }

    if let Some(app) = extra(app_id)? {
        return Ok(ResolvedApp::Stopped(ResolvedStoppedApp::new(
            SharedSageApp::new(app),
            guard,
        )));
    }

    match state.db.get_user_app(app_id).await {
        Ok(Some(app)) => {
            return Ok(ResolvedApp::Stopped(ResolvedStoppedApp::new(
                SharedSageApp::new(SageApp::User(app)),
                guard,
            )));
        }
        Ok(None) => {}
        Err(err) => {
            return Err(ResolveError::BuildFailed(format!(
                "failed to read installed app {app_id}: {err}"
            )));
        }
    }

    if let Some(app) = build_builtin_system_app(app_id).map_err(|err| {
        ResolveError::BuildFailed(format!(
            "failed to resolve builtin system app {app_id}: {err}"
        ))
    })? {
        return Ok(ResolvedApp::Stopped(ResolvedStoppedApp::new(
            SharedSageApp::new(app),
            guard,
        )));
    }

    if let Some(app) = build_builtin_test_app(app_id).map_err(|err| {
        ResolveError::BuildFailed(format!(
            "failed to resolve builtin sandbox app {app_id}: {err}"
        ))
    })? {
        return Ok(ResolvedApp::Stopped(ResolvedStoppedApp::new(
            SharedSageApp::new(app),
            guard,
        )));
    }

    Err(ResolveError::NotFound(format!(
        "failed to resolve app {app_id}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(value: &str) -> Url {
        Url::parse(value).expect("test URL should be valid")
    }

    #[test]
    fn allows_the_native_custom_protocol_origin() {
        assert!(is_allowed_app_origin_url(
            &url("sage-system-app://task-manager/index.html"),
            "sage-system-app",
            "task-manager",
        ));
    }

    #[test]
    fn allows_the_webview2_mapped_origin_only_on_windows() {
        assert_eq!(
            is_allowed_app_origin_url(
                &url("https://sage-system-app.task-manager/index.html"),
                "sage-system-app",
                "task-manager",
            ),
            cfg!(target_os = "windows")
        );
    }

    #[test]
    fn webview2_mapping_matches_only_the_exact_origin() {
        assert!(is_webview2_mapped_app_origin(
            &url("https://sage-system-app.task-manager/index.html"),
            "sage-system-app",
            "task-manager"
        ));

        for candidate in [
            "http://sage-system-app.task-manager/index.html",
            "http://sage-system-app.task-manager.invalid/index.html",
            "https://sage-system-app.other/index.html",
            "https://other.task-manager/index.html",
            "https://sage-system-app.task-manager:444/index.html",
        ] {
            assert!(
                !is_webview2_mapped_app_origin(&url(candidate), "sage-system-app", "task-manager"),
                "unexpectedly allowed {candidate}"
            );
        }
    }
}
