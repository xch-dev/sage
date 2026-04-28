use crate::lifecycle::read_installed_app_by_id;
use crate::runtime::{SageAppRuntimeKind, app_id_from_webview_label};
use crate::sandbox::builtin_runtime_apps_root;
use crate::types::SageAppCommon;
use crate::{AppsHostState, security::build_app_csp};
use anyhow::{Result as AnyResult, anyhow};
use std::fs;
use std::path::PathBuf;
use tauri::http::{Response, StatusCode};
use tauri::{Manager, State, UriSchemeContext, Wry};

pub const RUNTIME_APPS_PREFIX: &str = "/__sage/runtime-apps/";

pub fn handle_user_app_protocol_request(
    ctx: &UriSchemeContext<'_, Wry>,
    request: &tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = (|| -> anyhow::Result<_> {
        let webview_label = ctx.webview_label();

        let (runtime_kind, app_id) = app_id_from_webview_label(webview_label)
            .ok_or_else(|| anyhow!("invalid webview label"))?;

        if runtime_kind != SageAppRuntimeKind::User {
            anyhow::bail!("not a user runtime");
        }

        let app_handle = ctx.app_handle();
        let base_path = app_handle.path().app_data_dir()?;

        let app = read_installed_app_by_id(&base_path, app_id)?;

        if request.uri().host() != Some(app.common().origin_id()) {
            anyhow::bail!("host mismatch");
        }

        handle_app_protocol_request(app.common(), request)
    })();

    result.unwrap_or_else(|_| not_found_response())
}

pub fn handle_system_app_protocol_request(
    ctx: &UriSchemeContext<'_, Wry>,
    request: &tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = (|| {
        let webview_label = ctx.webview_label();

        let (runtime_kind, app_id) = app_id_from_webview_label(webview_label)
            .ok_or_else(|| anyhow!("invalid webview label"))?;

        if runtime_kind != SageAppRuntimeKind::System {
            anyhow::bail!("not a system runtime");
        }
        let state: State<'_, AppsHostState> = ctx.app_handle().state();
        let app = state
            .system_apps
            .get(app_id)
            .ok_or_else(|| anyhow!("unknown system app"))?;

        if request.uri().host() != Some(app.common().origin_id()) {
            anyhow::bail!("host mismatch");
        }

        handle_app_protocol_request(app.common(), request)
    })();

    result.unwrap_or_else(|_| not_found_response())
}

fn handle_app_protocol_request(
    app_common: &SageAppCommon,
    request: &tauri::http::Request<Vec<u8>>,
) -> AnyResult<Response<Vec<u8>>> {
    let request_path = request.uri().path();

    let file_path = match resolve_runtime_app_file(request_path)? {
        Some(file_path) => file_path,
        None => app_common
            .active_snapshot()
            .resolve_file_path(request_path)?,
    };

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "no-store")
        .header("Content-Security-Policy", build_app_csp(app_common))
        .header("X-Content-Type-Options", "nosniff")
        .body(fs::read(&file_path)?)
        .map_err(|err| anyhow!("failed to build app protocol response: {err}"))
}

fn resolve_runtime_app_file(request_path: &str) -> AnyResult<Option<PathBuf>> {
    let Some(relative_path) = request_path.strip_prefix(RUNTIME_APPS_PREFIX) else {
        return Ok(None);
    };

    let runtime_root = builtin_runtime_apps_root();
    let request_path = format!("/{relative_path}");

    Ok(Some(crate::lifecycle::read_snapshot_file(
        &runtime_root,
        &request_path,
    )?))
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body("Not found".to_string().into_bytes())
        .expect("failed to build error response")
}
