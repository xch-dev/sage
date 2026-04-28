use std::collections::BTreeMap;

use crate::lifecycle::read_installed_app_by_id;
use crate::runtime::state::SageAppRuntimeKind;
use crate::runtime::webview_locator::get_webview_in_sage_window;
use crate::sandbox::build_builtin_test_app;
use crate::system_apps::build_builtin_system_app;
use crate::types::SageApp;
use tauri::{AppHandle, Manager};
use url::Url;

pub fn app_id_from_webview_label(label: &str) -> Option<(SageAppRuntimeKind, &str)> {
    if let Some(app_id) = label.strip_prefix("app-inline-") {
        return Some((SageAppRuntimeKind::User, app_id));
    }

    if let Some(app_id) = label.strip_prefix("system-app-inline-") {
        return Some((SageAppRuntimeKind::System, app_id));
    }

    None
}

pub fn runtime_kind_for_app(app: &SageApp) -> SageAppRuntimeKind {
    match app {
        SageApp::User(_) => SageAppRuntimeKind::User,
        SageApp::System(_) => SageAppRuntimeKind::System,
    }
}

pub fn protocol_scheme_for_runtime_kind(runtime_kind: SageAppRuntimeKind) -> &'static str {
    match runtime_kind {
        SageAppRuntimeKind::User => "sage-app",
        SageAppRuntimeKind::System => "sage-system-app",
    }
}

pub fn is_allowed_app_url(url: &Url, origin_id: &str, runtime_kind: SageAppRuntimeKind) -> bool {
    url.scheme() == protocol_scheme_for_runtime_kind(runtime_kind)
        && url.host_str() == Some(origin_id)
}

pub fn build_entry_src(
    app: &SageApp,
    path: Option<String>,
    query: BTreeMap<String, String>,
) -> String {
    let runtime_kind = runtime_kind_for_app(app);
    let scheme = protocol_scheme_for_runtime_kind(runtime_kind);
    let entry_path = path.unwrap_or_else(|| format!("/{}", app.entry_file()));

    let mut url = Url::parse(&format!("{scheme}://{}{}", app.origin_id(), entry_path))
        .expect("failed to build app entry URL");

    for (key, value) in query {
        url.query_pairs_mut().append_pair(&key, &value);
    }

    url.to_string()
}

pub fn resolve_app(app: &AppHandle, app_id: &str) -> Result<SageApp, String> {
    let base_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    if let Ok(app) = read_installed_app_by_id(&base_path, app_id) {
        return Ok(SageApp::User(app));
    }

    if let Some(app) = build_builtin_system_app(app_id)
        .map_err(|err| format!("failed to resolve builtin system app {app_id}: {err}"))?
    {
        return Ok(app);
    }

    build_builtin_test_app(app_id)
        .map_err(|err| format!("failed to resolve builtin sandbox app {app_id}: {err}"))?
        .ok_or_else(|| format!("failed to resolve app {app_id}"))
}

pub(crate) fn assert_bridge_origin(
    app: &AppHandle,
    webview_label: &String,
) -> Result<(String, SageAppRuntimeKind), String> {
    let (runtime_kind, app_id) = app_id_from_webview_label(webview_label)
        .ok_or_else(|| format!("invalid app runtime label: {webview_label}"))?;

    let app_id = app_id.to_string();

    let resolved = resolve_app(app, &app_id)?;
    let expected_runtime_kind = runtime_kind_for_app(&resolved);

    if runtime_kind != expected_runtime_kind {
        return Err(format!(
            "bridge denied for {webview_label}: runtime kind mismatch (label={runtime_kind:?}, app={expected_runtime_kind:?})"
        ));
    }

    let app_webview = get_webview_in_sage_window(app, webview_label)?;

    let current_url = app_webview
        .url()
        .map_err(|e| format!("failed to read current webview url: {e}"))?;

    if !is_allowed_app_url(&current_url, resolved.origin_id(), runtime_kind) {
        return Err(format!(
            "bridge denied for {webview_label}: current url {} is outside {}://{}/...",
            current_url,
            protocol_scheme_for_runtime_kind(runtime_kind),
            resolved.origin_id()
        ));
    }

    Ok((app_id, runtime_kind))
}
