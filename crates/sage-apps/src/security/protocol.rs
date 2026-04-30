use crate::runtime::{app_id_from_webview_label, find_runtime_by_app_id_optional};
use crate::sandbox::builtin_runtime_apps_root;
use crate::types::SharedSageApp;
use crate::security::build_app_csp;
use anyhow::{Result as AnyResult, anyhow};
use std::fs;
use std::path::PathBuf;
use tauri::http::{Response, StatusCode};
use tauri::{Manager, UriSchemeContext, Wry};

pub const RUNTIME_APPS_PREFIX: &str = "/__sage/runtime-apps/";

pub async fn handle_user_app_protocol_request(
    ctx: &UriSchemeContext<'_, Wry>,
    request: &tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let webview_label = ctx.webview_label();

        let app_id = app_id_from_webview_label(webview_label)
            .ok_or_else(|| anyhow!("invalid webview label"))?;

        let runtime = find_runtime_by_app_id_optional(&ctx.app_handle().state(), app_id).await
            .ok_or_else(|| anyhow!("unknown app"))?;

        if !runtime.is_user_app() {
            anyhow::bail!("not a user runtime");
        }

        let app = runtime.app();

        if request.uri().host() != Some(&app.origin_id()) {
            anyhow::bail!("host mismatch");
        }

        let is_sandbox_test = app.with(|app| app.common().is_sandbox_test());

        match handle_app_protocol_request(&app, request) {
            Ok(response) => Ok(response),
            Err(err) if is_sandbox_test => Ok(protocol_error_response("sage-app", &err)),
            Err(_) => Ok(not_found_response()),
        }
    }.await;

    result.unwrap_or_else(|_| not_found_response())
}

pub async fn handle_system_app_protocol_request(
    ctx: &UriSchemeContext<'_, Wry>,
    request: &tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let webview_label = ctx.webview_label();

        let app_id = app_id_from_webview_label(webview_label)
            .ok_or_else(|| anyhow!("invalid webview label"))?;

        let runtime = find_runtime_by_app_id_optional(&ctx.app_handle().state(), app_id).await
            .ok_or_else(|| anyhow!("unknown app"))?;

        if !runtime.is_system_app() {
            anyhow::bail!("not a system runtime");
        }
        let app = runtime.app();

        if request.uri().host() != Some(&app.origin_id()) {
            anyhow::bail!("host mismatch");
        }

        handle_app_protocol_request(&app, request)
            .map_err(|err| anyhow!("sage-system-app error: {err}"))
    }.await;

    result.unwrap_or_else(|err| protocol_error_response("sage-system-app", &err))
}

fn handle_app_protocol_request(
    app: &SharedSageApp,
    request: &tauri::http::Request<Vec<u8>>,
) -> AnyResult<Response<Vec<u8>>> {
    let request_path = request.uri().path();

    let file_path = match resolve_runtime_app_file(request_path)? {
        Some(file_path) => file_path,
        None => {
            let file_path = app.with(|app| app.active_snapshot()
                .resolve_file_path(request_path));

            file_path?
        }
    };

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "no-store")
        .header("Content-Security-Policy", build_app_csp(app))
        .header("X-Content-Type-Options", "nosniff")
        .body(fs::read(&file_path)?)
        .map_err(|err| anyhow!("failed to build app protocol response: {err}"))
}

fn resolve_runtime_app_file(request_path: &str) -> AnyResult<Option<PathBuf>> {
    if let Some(relative_path) = request_path.strip_prefix(RUNTIME_APPS_PREFIX) {
        let runtime_root = builtin_runtime_apps_root();
        let request_path = format!("/{relative_path}");

        return Ok(Some(crate::lifecycle::read_snapshot_file(
            &runtime_root,
            &request_path,
        )?));
    }

    Ok(None)
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body("Not found".to_string().into_bytes())
        .expect("failed to build error response")
}

fn protocol_error_response(prefix: &str, err: &anyhow::Error) -> Response<Vec<u8>> {
    Response::builder()
        .status(500)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(format!("{prefix} error: {err}").into_bytes())
        .expect("failed to build protocol error response")
}
