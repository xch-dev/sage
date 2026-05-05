use crate::AppsHostState;
use crate::bridge::bridge_request::{process_system, process, process_after_approval};
use crate::bridge::{
    ResolveBridgeApprovalArgs, RustBridgeInvokeResult, RustBridgeRequest,
};
use crate::capabilities::user_registry;
use crate::host::AppState;
use crate::types::SageAppCapabilityDefinitionView;
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
) -> Result<RustBridgeInvokeResult, String> { process_system(app, webview, app_state, request).await }

#[tauri::command]
#[specta::specta]
pub async fn apps_resolve_bridge_approval(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    apps_state: State<'_, AppsHostState>,
    args: ResolveBridgeApprovalArgs,
) -> Result<(), String> { process_after_approval(&app_handle, &app_state, &apps_state, args).await }

#[tauri::command]
#[specta::specta]
pub fn get_user_capability_definitions() -> Result<Vec<SageAppCapabilityDefinitionView>, String> {
    Ok(user_registry().values().copied().map(Into::into).collect())
}
