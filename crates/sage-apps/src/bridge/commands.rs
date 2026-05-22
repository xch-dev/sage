use crate::bridge::bridge_request::{process, process_system};
use crate::bridge::{RustBridgeInvokeResult, RustBridgeRequest};
use crate::host::AppState;
use tauri::{AppHandle, State, Webview};

#[tauri::command]
#[specta::specta]
pub async fn apps_invoke_bridge(
    app: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    process(app, webview, app_state, request).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_invoke_system_bridge(
    app: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    process_system(app, webview, app_state, request).await
}
