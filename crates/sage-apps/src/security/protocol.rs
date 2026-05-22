use crate::host::AppState;
use crate::runtime::{app_id_from_webview_label, resolve_running_app};
use crate::security::build_app_csp;
use crate::types::{ResolvedRunningApp, SharedSageApp};
use anyhow::{Result as AnyResult, anyhow};
use std::fs;
use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

pub async fn handle_user_app_protocol_request(
    app_handle: AppHandle,
    webview_label: String,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let runtime = get_protocol_request_runtime(&app_handle, &webview_label).await?;

        let app = runtime.into_app();
        if !app.is_user_app() {
            anyhow::bail!("not a user runtime");
        }

        let is_sandbox_test = app.with(|app| app.common().is_sandbox_test());

        match handle_app_protocol_request(&app_handle, &app, &request).await {
            Ok(response) => Ok(response),
            Err(err) if is_sandbox_test => Ok(protocol_error_response("sage-app", &err)),
            Err(_) => Ok(not_found_response()),
        }
    }
    .await;

    result.unwrap_or_else(|_| not_found_response())
}

pub async fn handle_system_app_protocol_request(
    app_handle: AppHandle,
    webview_label: String,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let runtime = get_protocol_request_runtime(&app_handle, &webview_label).await?;
        let app = runtime.into_app();
        if !app.is_system_app() {
            anyhow::bail!("not a system runtime");
        }

        handle_app_protocol_request(&app_handle, &app, &request)
            .await
            .map_err(|err| anyhow!("sage-system-app error: {err}"))
    }
    .await;

    result.unwrap_or_else(|err| protocol_error_response("sage-system-app", &err))
}

async fn get_protocol_request_runtime(
    app_handle: &AppHandle,
    webview_label: &str,
) -> AnyResult<ResolvedRunningApp> {
    let app_id =
        app_id_from_webview_label(webview_label).ok_or_else(|| anyhow!("invalid webview label"))?;

    resolve_running_app(&app_handle.state(), app_id)
        .await
        .map_err(|_| anyhow!("failed to find runtime for app {app_id}"))
}

async fn handle_app_protocol_request(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    request: &Request<Vec<u8>>,
) -> AnyResult<Response<Vec<u8>>> {
    if request.uri().host() != Some(&app.origin_id()) {
        anyhow::bail!("host mismatch");
    }

    if request.headers().contains_key("Service-Worker") {
        anyhow::bail!("Service worker forbidden");
    }

    let request_path = request.uri().path();

    let file_path = app.with(|app| app.active_snapshot().resolve_file_path(request_path))?;

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    let network_id = active_network_id(app_handle).await?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "no-store")
        .header("Content-Security-Policy", build_app_csp(app, &network_id))
        .header("X-Content-Type-Options", "nosniff")
        .body(fs::read(&file_path)?)
        .map_err(|err| anyhow!("failed to build app protocol response: {err}"))
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body("Not found".to_string().into_bytes())
        .expect("failed to build error response")
}

fn protocol_error_response(prefix: &str, err: &anyhow::Error) -> Response<Vec<u8>> {
    tracing::error!(prefix, error = %err, "protocol request failed");

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(format!("{prefix} error: {err}").into_bytes())
        .expect("failed to build protocol error response")
}

async fn active_network_id(app_handle: &AppHandle) -> AnyResult<String> {
    use std::time::Duration;
    use tokio::time::timeout;

    const TIMEOUT: Duration = Duration::from_millis(500);

    let state = app_handle.state::<AppState>();

    let sage = timeout(TIMEOUT, state.lock())
        .await
        .map_err(|_| anyhow!("active network id unavailable because Sage state is locked"))?;

    Ok(sage.network_id())
}
