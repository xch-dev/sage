use crate::AppsHostState;
use crate::bridge::bridge_request::{execute_bridge_request, process};
use crate::bridge::event_emit::emit_bridge_response_to_source;
use crate::bridge::state::{get_pending_approval, remove_pending_approval};
use crate::bridge::{
    ResolveBridgeApprovalArgs, RustBridgeInvokeResult, RustBridgeRequest, RustBridgeResponse,
    response_channel_for_runtime_kind,
};
use crate::capabilities::user_registry;
use crate::host::AppState;
use crate::runtime::state::types::SageAppRuntimeKind;
use crate::runtime::{assert_bridge_origin, resolve_app};
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
    process(app, webview, app_state, request, SageAppRuntimeKind::User).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_invoke_system_bridge(
    app: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    process(app, webview, app_state, request, SageAppRuntimeKind::System).await
}

#[tauri::command]
#[specta::specta]
pub async fn apps_resolve_bridge_approval(
    app: AppHandle,
    app_state: State<'_, AppState>,
    apps_state: State<'_, AppsHostState>,
    args: ResolveBridgeApprovalArgs,
) -> Result<(), String> {
    let pending = get_pending_approval(&apps_state, &args.approval_id).await?;
    remove_pending_approval(&apps_state, &args.approval_id).await;

    let (app_id, runtime_kind) = assert_bridge_origin(&app, &pending.app_webview_label)?;
    let app_model = resolve_app(&app, &app_id)?;

    let response = if args.approved {
        execute_bridge_request(
            &app,
            &app_state,
            &app_model,
            &pending.app_webview_label,
            &pending.request,
        )
        .await
    } else {
        RustBridgeResponse::error(
            response_channel_for_runtime_kind(runtime_kind),
            &pending.request.id,
            "user_denied",
            args.reason
                .unwrap_or_else(|| "User denied the request".to_string()),
        )
    };

    emit_bridge_response_to_source(&app, &pending.app_webview_label, &response)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_user_capability_definitions() -> Result<Vec<SageAppCapabilityDefinitionView>, String> {
    Ok(user_registry().values().copied().map(Into::into).collect())
}
