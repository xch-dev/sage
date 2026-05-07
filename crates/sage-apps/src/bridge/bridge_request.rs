use crate::AppsHostState;
use crate::capabilities::list::{BridgeCapability, SystemBridgeCapability, UserBridgeCapability};
use crate::bridge::methods::BridgeMethodCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::registry::{BridgeRegistry, BridgeRegistryKind};
use crate::bridge::state::{ensure_approval_expiry_loop, get_pending_approval, list_pending_approvals, remove_pending_approval, write_pending_approval};
use crate::bridge::{emit_system_runtime_event_to_listeners, BridgeOrigin, ResolveBridgeApprovalArgs, RustBridgeApprovalRequest, RustBridgeInvokeResult, RustBridgeRequest, RustBridgeResponse};
use crate::capabilities::{get_system_capability_definition, get_user_capability_definition};
use crate::host::AppState;
use crate::runtime::{resolve_app, start_bridge_approval_runtime, sync_bridge_approval_runtime, SharedImpostorRuntime};
use crate::types::SharedSageApp;
use tauri::{AppHandle, Manager, State, Webview};
use crate::bridge::event_emit::emit_bridge_response_to_app;
use crate::bridge::methods::system::BridgeApprovalsChangedEvent;
use crate::lifecycle::{ensure_app_is_enabled_for_scope};
use crate::security::assert_bridge_origin;

pub(super) async fn process(
    app_handle: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    if let Err(result) = assert_bridge_version(&request) {
        return Ok(result);
    }

    let webview_label = webview.label().to_string();

    let origin = match assert_bridge_origin(&app_handle, &webview_label).await {
        Ok(origin) => origin,
        Err(err) => {
            return Ok(RustBridgeInvokeResult::error(
                &request.id,
                "permission_denied",
                format!("Bridge origin denied: {err}"),
            ));
        }
    };

    process_shared(&app_handle, &app_state, &origin, BridgeRegistryKind::User, &request, false).await
}

pub(super) async fn process_system(
    app_handle: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    if let Err(result) = assert_bridge_version(&request) {
        return Ok(result);
    }
    let webview_label = webview.label().to_string();

    let origin = match assert_system_bridge_origin(&app_handle, &webview_label).await {
        Ok(origin) => origin,
        Err(err) => {
            return Ok(RustBridgeInvokeResult::error(
                &request.id,
                "permission_denied",
                format!("Bridge origin denied: {err}"),
            ));
        }
    };

    process_shared(&app_handle, &app_state, &origin, BridgeRegistryKind::System, &request, false).await
}

pub(super) async fn process_after_approval(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    apps_state: &State<'_, AppsHostState>,
    args: ResolveBridgeApprovalArgs,
) -> Result<(), String> {
    let pending = get_pending_approval(apps_state, &args.approval_id).await?;
    remove_pending_approval(apps_state, &args.approval_id).await;

    sync_bridge_approval_runtime(app_handle, apps_state).await?;

    let approvals_changed_event = BridgeApprovalsChangedEvent::new_from_list(
        list_pending_approvals(apps_state).await,
    );

    emit_system_runtime_event_to_listeners(
        app_handle,
        apps_state,
        approvals_changed_event,
    ).await;

    let app = resolve_app(app_handle, &pending.app_id).await
        .map_err(|err| format!("Failed to resolve app: {err}"))?;

    let origin = assert_bridge_origin(app_handle, &app.with_app(SharedSageApp::webview_label)).await?;

    let invoke_result = if args.approved {
        process_shared(app_handle, app_state, &origin, pending.registry_kind, &pending.request, true).await?
    } else {
        RustBridgeInvokeResult::error(
            &pending.request.id,
            "user_denied",
            args.reason
                .unwrap_or_else(|| "User denied the request".to_string()),
        )
    };

    emit_bridge_response_to_app(app_handle, &origin.app, &invoke_result.try_into()?).await?;
    Ok(())
}

async fn process_shared(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    origin: &BridgeOrigin,
    registry_kind: BridgeRegistryKind,
    request: &RustBridgeRequest,
    approved: bool,
) -> Result<RustBridgeInvokeResult, String> {
    let registry = BridgeRegistry::new(registry_kind);

    let app = &origin.app;
    let impostor_runtime = &origin.impostor_runtime;
    if let Err(err) = ensure_app_is_enabled_for_scope(app_state, app).await {
        return Ok(RustBridgeInvokeResult::error(
            &request.id,
            "app_not_enabled_for_scope",
            err,
        ));
    }

    let method = match assert_method(&registry, request) {
        Ok(method) => method,
        Err(response) => return Ok(response.into())
    };

    let authority_app = origin
        .impostor_runtime
        .as_ref().map_or_else(|| app.clone_for_runtime_owner(), SharedImpostorRuntime::impostor_app);

    match method.capability() {
        BridgeMethodCapability::Ungated => {}

        BridgeMethodCapability::Required(capability) => {
            if let Err(response) = verify_capability(&authority_app, request, capability) {
                return Ok(response.into());
            }
        }
    }

    if approved {
        let response =
            execute_bridge_request(app_handle, app_state, origin, registry, request).await;

        return Ok(response.into());
    }

    match method.approval_request(
        BridgeContext {
            app,
            impostor_runtime,
        },
        request,
    ) {
        Ok(Some(approval)) => {
            request_approval(app_handle, app.id(), registry_kind, approval, request).await?;
            Ok(RustBridgeInvokeResult::Pending {})
        }
        Ok(None) => {
            let response =
                execute_bridge_request(app_handle, app_state, origin, registry, request).await;

            Ok(response.into())
        }
        Err(err) => {
            Ok(RustBridgeInvokeResult::error(
                &request.id,
                err.code,
                err.message,
            ))
        }
    }
}

async fn execute_bridge_request(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    origin: &BridgeOrigin,
    registry: BridgeRegistry,
    request: &RustBridgeRequest,
) -> RustBridgeResponse {
    let method = match assert_method(&registry, request) {
        Ok(method) => method,
        Err(response) => return response
    };

    let result = method
        .handle(
            BridgeContext { app: &origin.app, impostor_runtime: &origin.impostor_runtime},
            BridgeTools {
                app_handle,
                app_state,
                host_state: &app_handle.state::<AppsHostState>(),
            },
            request,
        )
        .await;

    match result {
        Ok(value) => match erased_serde::serialize(&*value, serde_json::value::Serializer) {
            Ok(value) => RustBridgeResponse::success(&request.id, &value),
            Err(err) => RustBridgeResponse::error(
                &request.id,
                "internal_error",
                format!("failed to encode {} result: {err}", method.name()),
            ),
        },
        Err(err) => RustBridgeResponse::error(&request.id, err.code, err.message),
    }
}

async fn request_approval(
    app_handle: &AppHandle,
    app_id: String,
    registry_kind: BridgeRegistryKind,
    approval: RustBridgeApprovalRequest,
    request: &RustBridgeRequest,
) -> Result<(), String> {
    let apps_state = app_handle.state::<AppsHostState>();
    write_pending_approval(
        &apps_state,
        app_id.clone(),
        registry_kind,
        &approval,
        request,
    )
        .await;

    ensure_approval_expiry_loop(app_handle, &apps_state).await;

    let approvals_changed_event = BridgeApprovalsChangedEvent::new_from_list(
        list_pending_approvals(&apps_state).await
    );
    emit_system_runtime_event_to_listeners(app_handle, &apps_state, approvals_changed_event).await;

    start_bridge_approval_runtime(
        app_handle,
        &apps_state,
        Vec::from([app_id]),
    ).await?;

    Ok(())
}

fn verify_capability(
    app: &SharedSageApp,
    request: &RustBridgeRequest,
    capability: BridgeCapability,
) -> Result<(), RustBridgeResponse> {
    match capability {
        BridgeCapability::User(capability) => {
            let definition = get_user_capability_definition(capability);

            verify_user_capability(
                app,
                request,
                capability,
                definition.flags().shared_with_app(),
            )
        }

        BridgeCapability::System(capability) => {
            let definition = get_system_capability_definition(capability);

            verify_system_capability(
                app,
                request,
                capability,
                definition.flags().shared_with_app(),
            )
        }
    }
}

fn verify_user_capability(
    app: &SharedSageApp,
    request: &RustBridgeRequest,
    capability: UserBridgeCapability,
    shared_with_app: bool,
) -> Result<(), RustBridgeResponse> {
    if !shared_with_app {
        return Err(RustBridgeResponse::error(
            &request.id,
            "permission_denied",
            format!("Capability {} is not shared with apps", capability.key()),
        ));
    }

    let effective_capabilities = app.with(|app| {
        app.common()
            .requested_permissions()
            .capabilities()
            .resolve_effective_grants(
                app.common()
                    .granted_permissions()
                    .capabilities()
                    .copied(),
            )
    });

    if !effective_capabilities.contains(&capability) {
        return Err(RustBridgeResponse::error(
            &request.id,
            "permission_denied",
            format!("Permission denied for {}", capability.key()),
        ));
    }

    Ok(())
}

fn verify_system_capability(
    app: &SharedSageApp,
    request: &RustBridgeRequest,
    capability: SystemBridgeCapability,
    shared_with_app: bool,
) -> Result<(), RustBridgeResponse> {
    if !shared_with_app {
        return Err(RustBridgeResponse::error(
            &request.id,
            "permission_denied",
            format!("Capability {} is not shared with apps", capability.key()),
        ));
    }

    let granted = app.with(|app| app
        .system_granted_permissions()
        .is_some_and(|permissions| permissions.capabilities().contains(&capability)));

    if !granted {
        return Err(RustBridgeResponse::error(
            &request.id,
            "permission_denied",
            format!("Permission denied for {}", capability.key()),
        ));
    }

    Ok(())
}

fn assert_method<'a>(
    registry: &'a BridgeRegistry,
    request: &RustBridgeRequest,
) -> Result<&'a dyn BridgeMethod, RustBridgeResponse> {
    let Some(method) = registry.get(&request.method) else {
        return Err(RustBridgeResponse::error(
            &request.id,
            "method_not_found",
            format!("Unknown bridge method: {}", request.method),
        ));
    };

    Ok(method)
}

async fn assert_system_bridge_origin(
    app_handle: &AppHandle,
    webview_label: &String,
) -> Result<BridgeOrigin, String> {
    let origin = assert_bridge_origin(app_handle, webview_label).await?;

    if !origin.app.is_system_app() {
        return Err("origin app is not a system app".to_string());
    }

    Ok(origin)
}

fn assert_bridge_version(request: &RustBridgeRequest) -> Result<(), RustBridgeInvokeResult> {
    if let Some(version) = &request.bridge_version && version != "v1" {
        return Err(RustBridgeInvokeResult::error(
            &request.id,
            "unsupported_bridge_version",
            format!("Unsupported Sage bridge version: {version}"),
        ));
    }

    Ok(())
}
