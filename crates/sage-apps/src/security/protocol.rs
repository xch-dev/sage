use crate::runtime::{app_id_from_webview_label, resolve_possibly_impostor_running_app, PossiblyImpostorRuntime};
use crate::security::build_app_csp;
use anyhow::{Result as AnyResult, anyhow};
use std::fs;
use tauri::http::{Response, StatusCode};
use tauri::{Manager, UriSchemeContext, Wry};

pub async fn handle_user_app_protocol_request(
    ctx: &UriSchemeContext<'_, Wry>,
    request: &tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let webview_label = ctx.webview_label();

        let app_id = app_id_from_webview_label(webview_label)
            .ok_or_else(|| anyhow!("invalid webview label"))?;

        let runtime =
            resolve_possibly_impostor_running_app(&ctx.app_handle().state(), app_id)
                .await
                .map_err(|_| anyhow!("failed to find runtime for app {app_id}"))?;

        if !runtime.is_user_app() {
            anyhow::bail!("not a user runtime");
        }

        let identity_app = runtime.identity_app();

        if request.uri().host() != Some(&identity_app.origin_id()) {
            anyhow::bail!("host mismatch");
        }

        let is_sandbox_test = identity_app.with(|app| app.common().is_sandbox_test());

        match handle_app_protocol_request(&runtime, request) {
            Ok(response) => Ok(response),
            Err(err) if is_sandbox_test => Ok(protocol_error_response("sage-app", &err)),
            Err(_) => Ok(not_found_response()),
        }
    }
        .await;

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

        let runtime =
            resolve_possibly_impostor_running_app(&ctx.app_handle().state(), app_id)
                .await
                .map_err(|_| anyhow!("failed to find runtime for app {app_id}"))?;

        if !runtime.is_system_app() {
            anyhow::bail!("not a system runtime");
        }

        let identity_app = runtime.identity_app();

        if request.uri().host() != Some(&identity_app.origin_id()) {
            anyhow::bail!("host mismatch");
        }

        handle_app_protocol_request(&runtime, request)
            .map_err(|err| anyhow!("sage-system-app error: {err}"))
    }
        .await;

    result.unwrap_or_else(|err| protocol_error_response("sage-system-app", &err))
}

fn handle_app_protocol_request(
    runtime: &PossiblyImpostorRuntime,
    request: &tauri::http::Request<Vec<u8>>,
) -> AnyResult<Response<Vec<u8>>> {
    let identity_app = runtime.identity_app();
    let content_app = runtime.content_app();
    let request_path = request.uri().path();

    let file_path = content_app.with(|app| {
        app.active_snapshot().resolve_file_path(request_path)
    })?;

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "no-store")
        .header("Content-Security-Policy", build_app_csp(&identity_app))
        .header("X-Content-Type-Options", "nosniff")
        .body(fs::read(&file_path)?)
        .map_err(|err| anyhow!("failed to build app protocol response: {err}"))
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
