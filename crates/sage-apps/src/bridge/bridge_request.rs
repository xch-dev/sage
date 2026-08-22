use tauri::{AppHandle, Manager, State, Webview};

use crate::{
    AppState, AppsHostState, BridgeApprovalsChangedEvent, BridgeCapability, BridgeContext,
    BridgeMethod, BridgeMethodCapability, BridgeOrigin, BridgeRegistry, BridgeRegistryKind,
    BridgeTools, PendingBridgeApproval, ResolveBridgeApprovalArgs, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeInvokeResult, RustBridgeRequest, RustBridgeResponse,
    SharedSageApp, SystemBridgeCapability, UserBridgeCapability, assert_bridge_origin,
    emit_bridge_response_to_app, emit_system_runtime_event_to_listeners,
    ensure_app_is_enabled_for_scope, ensure_approval_expiry_loop, get_system_capability_definition,
    get_user_capability_definition, list_pending_approvals, resolve_app,
    start_bridge_approval_runtime, sync_bridge_approval_runtime, take_pending_approval,
    unix_timestamp_ms, write_pending_approval,
};

pub(crate) async fn process(
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

    process_shared(
        &app_handle,
        &app_state,
        &origin,
        BridgeRegistryKind::User,
        &request,
        false,
    )
    .await
}

pub(crate) async fn process_system(
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

    process_shared(
        &app_handle,
        &app_state,
        &origin,
        BridgeRegistryKind::System,
        &request,
        false,
    )
    .await
}

pub(crate) async fn process_after_approval(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    apps_state: &State<'_, AppsHostState>,
    args: ResolveBridgeApprovalArgs,
) -> Result<(), String> {
    let pending = take_pending_approval(apps_state, &args.approval_id)
        .await
        .ok_or_else(|| format!("No pending approval with id {}", args.approval_id))?;

    sync_bridge_approval_runtime(app_handle, apps_state).await?;

    let approvals_changed_event =
        BridgeApprovalsChangedEvent::new_from_list(list_pending_approvals(apps_state).await);

    emit_system_runtime_event_to_listeners(app_handle, apps_state, approvals_changed_event).await;

    let app = resolve_app(app_handle, &pending.app_id)
        .await
        .map_err(|err| format!("Failed to resolve app: {err}"))?;

    let origin =
        assert_bridge_origin(app_handle, &app.with_app(SharedSageApp::webview_label)).await?;

    let invoke_result = if !args.approved {
        RustBridgeInvokeResult::error(
            &pending.request.id,
            "user_denied",
            args.reason
                .unwrap_or_else(|| "User denied the request".to_string()),
        )
    } else if unix_timestamp_ms() as u64 > pending.expires_at_ms {
        RustBridgeInvokeResult::error(
            &pending.request.id,
            "approval_timeout",
            "Approval expired before it was resolved".to_string(),
        )
    } else if wallet_binding_violated(app_state, &pending).await {
        RustBridgeInvokeResult::error(
            &pending.request.id,
            "wallet_changed",
            "Active wallet changed since the approval was requested".to_string(),
        )
    } else {
        process_shared(
            app_handle,
            app_state,
            &origin,
            pending.registry_kind,
            &pending.request,
            true,
        )
        .await?
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
    if let Err(err) = ensure_app_is_enabled_for_scope(app_state, app).await {
        return Ok(RustBridgeInvokeResult::error(
            &request.id,
            "app_not_enabled_for_scope",
            err,
        ));
    }

    let method = match assert_method(&registry, request) {
        Ok(method) => method,
        Err(response) => return Ok(response.into()),
    };

    match method.capability() {
        BridgeMethodCapability::Ungated => {}

        BridgeMethodCapability::Required(capability) => {
            if let Err(response) = verify_capability(&origin.app, request, capability) {
                return Ok(response.into());
            }
        }
    }

    if approved {
        let response =
            execute_bridge_request(app_handle, app_state, origin, registry, request).await;

        return Ok(response.into());
    }

    match method
        .prepare_approval(
            BridgeContext { app },
            BridgeTools {
                app_handle,
                app_state,
                host_state: &app_handle.state::<AppsHostState>(),
            },
            request,
        )
        .await
    {
        Ok(Some(approval)) => {
            request_approval(
                app_handle,
                app_state,
                app.id(),
                registry_kind,
                approval,
                request,
            )
            .await?;
            Ok(RustBridgeInvokeResult::Pending {})
        }
        Ok(None) => {
            let response =
                execute_bridge_request(app_handle, app_state, origin, registry, request).await;

            Ok(response.into())
        }
        Err(err) => Ok(RustBridgeInvokeResult::error(
            &request.id,
            err.code,
            err.message,
        )),
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
        Err(response) => return response,
    };

    let result = method
        .handle(
            BridgeContext { app: &origin.app },
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

async fn active_wallet_fingerprint(app_state: &State<'_, AppState>) -> Option<u32> {
    app_state
        .lock()
        .await
        .wallet()
        .map(|wallet| wallet.fingerprint)
        .ok()
}

async fn wallet_binding_violated(
    app_state: &State<'_, AppState>,
    pending: &PendingBridgeApproval,
) -> bool {
    let requires_wallet_binding = matches!(
        pending.approval.body,
        RustBridgeApprovalBody::SendXch { .. }
            | RustBridgeApprovalBody::SignCoinSpends { .. }
            | RustBridgeApprovalBody::SignMessage { .. }
    );

    if !requires_wallet_binding {
        return false;
    }

    let Some(approved_fingerprint) = pending.approved_fingerprint else {
        return true;
    };

    active_wallet_fingerprint(app_state).await != Some(approved_fingerprint)
}

async fn request_approval(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    app_id: String,
    registry_kind: BridgeRegistryKind,
    approval: RustBridgeApprovalRequest,
    request: &RustBridgeRequest,
) -> Result<(), String> {
    let apps_state = app_handle.state::<AppsHostState>();
    let approved_fingerprint = active_wallet_fingerprint(app_state).await;

    write_pending_approval(
        &apps_state,
        app_id.clone(),
        registry_kind,
        &approval,
        request,
        approved_fingerprint,
    )
    .await;

    ensure_approval_expiry_loop(app_handle, &apps_state).await;

    let approvals_changed_event =
        BridgeApprovalsChangedEvent::new_from_list(list_pending_approvals(&apps_state).await);
    emit_system_runtime_event_to_listeners(app_handle, &apps_state, approvals_changed_event).await;

    start_bridge_approval_runtime(app_handle, &apps_state, Vec::from([app_id])).await?;

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
            .resolve_effective_grants(app.common().granted_permissions().capabilities().copied())
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

    let granted = app.with(|app| {
        app.system_granted_permissions()
            .is_some_and(|permissions| permissions.capabilities().contains(&capability))
    });

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
    if let Some(version) = &request.bridge_version
        && version != "v1"
    {
        return Err(RustBridgeInvokeResult::error(
            &request.id,
            "unsupported_bridge_version",
            format!("Unsupported Sage bridge version: {version}"),
        ));
    }

    Ok(())
}
