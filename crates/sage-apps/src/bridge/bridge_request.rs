use tauri::{AppHandle, Emitter, Manager, State, Webview};
use uuid::Uuid;
use crate::AppsHostState;
use crate::bridge::capabilities::{BridgeCapability, SystemBridgeCapability, UserBridgeCapability};
use crate::bridge::{response_channel_for_runtime_kind, RustBridgeApprovalEvent, RustBridgeApprovalRequest, RustBridgeInvokeResult, RustBridgeRequest, RustBridgeResponse};
use crate::bridge::methods::{BridgeContext, BridgeTools};
use crate::bridge::methods::shared::BridgeMethodCapability;
use crate::bridge::registry::BridgeRegistry;
use crate::bridge::state::write_pending_approval;
use crate::host::AppState;
use crate::permissions::{get_system_capability_definition, get_user_capability_definition};
use crate::permissions::resolve_and_validate_effective_granted_capabilities;
use crate::runtime::{assert_bridge_origin, resolve_app};
use crate::runtime::state::types::SageAppRuntimeKind;
use crate::runtime::webview_locator::get_sage_webview;
use crate::types::SageApp;

pub async fn process(
    app: AppHandle,
    webview: Webview,
    app_state: State<'_, AppState>,
    request: RustBridgeRequest,
    expected_runtime_kind: SageAppRuntimeKind,
) -> Result<RustBridgeInvokeResult, String> {
    let expected_channel = response_channel_for_runtime_kind(expected_runtime_kind);

    if let Err(response) = validate_request_basics(&request, expected_channel) {
        return Ok(RustBridgeInvokeResult::Immediate { response });
    }

    let webview_label = webview.label().to_string();

    let (app_id, runtime_kind) = match assert_bridge_origin(app.clone(), webview_label.clone()) {
        Ok(value) => value,
        Err(err) => {
            return Ok(RustBridgeInvokeResult::Immediate {
                response: RustBridgeResponse::error(
                    expected_channel,
                    &request.id,
                    "permission_denied",
                    format!("Bridge origin denied: {err}"),
                ),
            });
        }
    };

    if runtime_kind != expected_runtime_kind {
        return Ok(RustBridgeInvokeResult::Immediate {
            response: RustBridgeResponse::error(
                expected_channel,
                &request.id,
                "permission_denied",
                "This bridge is not available for this runtime kind",
            ),
        });
    }

    let app_model = resolve_app(&app, &app_id)?;
    let registry = BridgeRegistry::new_for_app(&app_model);

    let Some(method) = registry.get(&request.method) else {
        return Ok(RustBridgeInvokeResult::Immediate {
            response: RustBridgeResponse::error(
                expected_channel,
                &request.id,
                "method_not_found",
                format!("Unknown bridge method: {}", request.method),
            ),
        });
    };

    match method.capability() {
        BridgeMethodCapability::Ungated => {}
        BridgeMethodCapability::Required(capability) => {
            if let Err(response) =
                verify_capability(&app_model, &request, capability)
            {
                return Ok(RustBridgeInvokeResult::Immediate { response });
            }
        }
    }

    match method.approval_request(
        BridgeContext {
            app: &app_model,
            source_label: &webview_label,
        },
        &request,
    ) {
        Ok(Some(approval)) => {
            let approval_id = Uuid::new_v4().to_string();

            let apps_state = app.state::<AppsHostState>();
            write_pending_approval(&apps_state, &approval_id, &app_model, &webview_label, &request).await;

            emit_sage_approval_requested(&app, approval_id, approval)?;
            return Ok(RustBridgeInvokeResult::Pending {});
        }
        Ok(None) => {}
        Err(err) => {
            return Ok(RustBridgeInvokeResult::Immediate {
                response: RustBridgeResponse::error(
                    expected_channel,
                    &request.id,
                    err.code,
                    err.message,
                ),
            });
        }
    }

    let response = execute_bridge_request(
        &app,
        &app_state,
        &app_model,
        &webview_label,
        &request,
    )
        .await;

    Ok(RustBridgeInvokeResult::Immediate { response })
}

pub(crate) async fn execute_bridge_request(
    app_handle: &AppHandle,
    app_state: &State<'_, AppState>,
    app: &SageApp,
    source_label: &str,
    request: &RustBridgeRequest,
) -> RustBridgeResponse {
    let registry = BridgeRegistry::new_for_app(app);

    let Some(method) = registry.get(&request.method) else {
        return RustBridgeResponse::error(
            &request.channel,
            &request.id,
            "method_not_found",
            format!("Unknown bridge method: {}", request.method),
        );
    };

    let result = method
        .handle(
            BridgeContext { app, source_label },
            BridgeTools {
                app_handle,
                app_state,
                host_state: &app_handle.state::<AppsHostState>(),
            },
            request,
        )
        .await;

    match result {
        Ok(value) => {
            match erased_serde::serialize(&*value, serde_json::value::Serializer) {
                Ok(value) => RustBridgeResponse::success(&request.channel, &request.id, value),
                Err(err) => RustBridgeResponse::error(
                    &request.channel,
                    &request.id,
                    "internal_error",
                    format!("failed to encode {} result: {err}", method.name()),
                ),
            }
        }
        Err(err) => RustBridgeResponse::error(
            &request.channel,
            &request.id,
            err.code,
            err.message,
        ),
    }
}

fn verify_capability(
    app: &SageApp,
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
                definition.flags.shared_with_app,
            )
        }

        BridgeCapability::System(capability) => {
            let definition = get_system_capability_definition(capability);

            verify_system_capability(
                app,
                request,
                capability,
                definition.flags.shared_with_app,
            )
        }
    }
}

fn verify_user_capability(
    app: &SageApp,
    request: &RustBridgeRequest,
    capability: UserBridgeCapability,
    shared_with_app: bool,
) -> Result<(), RustBridgeResponse> {
    if !shared_with_app {
        return Err(RustBridgeResponse::error(
            &request.channel,
            &request.id,
            "permission_denied",
            format!("Capability {} is not shared with apps", capability.key()),
        ));
    }

    let effective_capabilities = match app {
        SageApp::User(user_app) => resolve_and_validate_effective_granted_capabilities(
            &user_app.common.requested_permissions.capabilities,
            &user_app.common.granted_permissions.capabilities,
        )
            .map_err(|err| {
                RustBridgeResponse::error(
                    &request.channel,
                    &request.id,
                    "internal_error",
                    format!("failed to resolve effective permissions: {err}"),
                )
            })?,
        SageApp::System(_) => app.granted_permissions().capabilities.clone(),
    };

    if !effective_capabilities.contains(&capability) {
        return Err(RustBridgeResponse::error(
            &request.channel,
            &request.id,
            "permission_denied",
            format!("Permission denied for {}", capability.key()),
        ));
    }

    Ok(())
}

fn verify_system_capability(
    app: &SageApp,
    request: &RustBridgeRequest,
    capability: SystemBridgeCapability,
    shared_with_app: bool,
) -> Result<(), RustBridgeResponse> {
    if !shared_with_app {
        return Err(RustBridgeResponse::error(
            &request.channel,
            &request.id,
            "permission_denied",
            format!("Capability {} is not shared with apps", capability.key()),
        ));
    }

    let granted = app
        .system_granted_permissions()
        .map(|permissions| permissions.capabilities.contains(&capability))
        .unwrap_or(false);

    if !granted {
        return Err(RustBridgeResponse::error(
            &request.channel,
            &request.id,
            "permission_denied",
            format!("Permission denied for {}", capability.key()),
        ));
    }

    Ok(())
}

fn validate_request_basics(
    request: &RustBridgeRequest,
    expected_channel: &str,
) -> Result<(), RustBridgeResponse> {
    if request.channel != expected_channel {
        return Err(RustBridgeResponse::error(
            expected_channel,
            &request.id,
            "invalid_request",
            "Invalid bridge channel",
        ));
    }

    if let Some(version) = &request.bridge_version {
        if version != "v1" {
            return Err(RustBridgeResponse::error(
                expected_channel,
                &request.id,
                "unsupported_bridge_version",
                format!("Unsupported Sage bridge version: {version}"),
            ));
        }
    }

    Ok(())
}

fn emit_sage_approval_requested(
    app: &AppHandle,
    approval_id: String,
    approval: RustBridgeApprovalRequest,
) -> Result<(), String> {
    get_sage_webview(app)?
        .emit(
            "apps:bridge-approval-requested",
            RustBridgeApprovalEvent {
                approval_id,
                approval,
            },
        )
        .map_err(|err| format!("failed to emit approval request event: {err}"))
}
