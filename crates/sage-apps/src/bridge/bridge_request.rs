use crate::AppsHostState;
use crate::bridge::capabilities::{BridgeCapability, SystemBridgeCapability, UserBridgeCapability};
use crate::bridge::methods::shared::BridgeMethodCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::registry::{BridgeRegistry, BridgeRegistryKind};
use crate::bridge::state::write_pending_approval;
use crate::bridge::{RustBridgeApprovalEvent, RustBridgeApprovalRequest, RustBridgeInvokeResult, RustBridgeRequest, RustBridgeResponse};
use crate::capabilities::{get_system_capability_definition, get_user_capability_definition};
use crate::host::AppState;
use crate::runtime::webview_locator::{get_sage_webview, get_webview_in_sage_window};
use crate::runtime::{app_id_from_webview_label, is_allowed_app_url, protocol_scheme_for_app, resolve_possibly_impostor_running_app, PossiblyImpostorRuntime, SharedImpostorRuntime};
use crate::types::{SageApp, SharedSageApp};
use tauri::{AppHandle, Emitter, Manager, State, Webview};
use uuid::Uuid;

pub(crate) struct BridgeOrigin {
    pub app: SharedSageApp,
    pub impostor_runtime: Option<SharedImpostorRuntime>,
}

pub async fn process_user(
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
        Ok(value) => value,
        Err(err) => {
            return Ok(RustBridgeInvokeResult::Immediate {
                response: RustBridgeResponse::error(
                    &request.id,
                    "permission_denied",
                    format!("Bridge origin denied: {err}"),
                ),
            });
        }
    };

    process_shared(app_handle, app_state, origin, BridgeRegistryKind::User, request).await
}

pub async fn process_system(
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
            return Ok(RustBridgeInvokeResult::Immediate {
                response: RustBridgeResponse::error(
                    &request.id,
                    "permission_denied",
                    format!("Bridge origin denied: {err}"),
                ),
            });
        }
    };

    process_shared(app_handle, app_state, origin, BridgeRegistryKind::System, request).await
}

async fn process_shared(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    origin: BridgeOrigin,
    registry_kind: BridgeRegistryKind,
    request: RustBridgeRequest,
) -> Result<RustBridgeInvokeResult, String> {
    let registry = BridgeRegistry::new(registry_kind);

    let app = &origin.app;
    let impostor_runtime = &origin.impostor_runtime;

    let method = match assert_method(&registry, &request) {
        Ok(method) => method,
        Err(response) => return Ok(RustBridgeInvokeResult::Immediate { response })
    };

    let authority_app = origin
        .impostor_runtime
        .as_ref().map_or_else(|| app.clone_for_runtime_owner(), SharedImpostorRuntime::impostor_app);

    match method.capability() {
        BridgeMethodCapability::Ungated => {}

        BridgeMethodCapability::Required(capability) => {
            if let Err(response) = verify_capability(&authority_app, &request, capability) {
                return Ok(RustBridgeInvokeResult::Immediate { response });
            }
        }
    }

    match method.approval_request(
        BridgeContext {
            app,
            impostor_runtime,
        },
        &request,
    ) {
        Ok(Some(approval)) => {
            let approval_id = Uuid::new_v4().to_string();

            let apps_state = app_handle.state::<AppsHostState>();
            write_pending_approval(
                &apps_state,
                &approval_id,
                app,
                &request,
                registry_kind,
            )
            .await;

            emit_sage_approval_requested(&app_handle, approval_id, approval)?;
            return Ok(RustBridgeInvokeResult::Pending {});
        }
        Ok(None) => {}
        Err(err) => {
            return Ok(RustBridgeInvokeResult::Immediate {
                response: RustBridgeResponse::error(
                    &request.id,
                    err.code,
                    err.message,
                ),
            });
        }
    }

    let response =
        execute_bridge_request(&app_handle, &app_state, &origin, registry, &request).await;

    Ok(RustBridgeInvokeResult::Immediate { response })
}

pub(crate) async fn execute_bridge_request(
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
        let effective = match app {
            SageApp::User(user_app) => user_app
                .common()
                .requested_permissions()
                .capabilities()
                .resolve_effective_grants(
                    user_app
                        .common()
                        .granted_permissions()
                        .capabilities()
                        .copied(),
                )
                .map_err(|err| {
                    RustBridgeResponse::error(
                        &request.id,
                        "internal_error",
                        format!("failed to resolve effective permissions: {err}"),
                    )
                })?,

            SageApp::System(_) => app.granted_permissions().capabilities().copied().collect(),
        };

        Ok(effective)
    })?;

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

pub(super) async fn assert_bridge_origin(
    app_handle: &AppHandle,
    webview_label: &String,
) -> Result<BridgeOrigin, String> {
    let app_id = app_id_from_webview_label(webview_label)
        .ok_or_else(|| format!("invalid app runtime label: {webview_label}"))?;

    let runtime = resolve_possibly_impostor_running_app(&app_handle.state(), app_id)
        .await
        .map_err(|_| format!("failed to find runtime for app {app_id}"))?;

    let app = runtime.identity_app();

    let impostor_runtime = match &runtime {
        PossiblyImpostorRuntime::Legit(_) => None,
        PossiblyImpostorRuntime::Impostor(runtime) => Some(runtime),
    };

    if !app.webview_label_matches(webview_label) {
        return Err(format!(
            "bridge denied for {webview_label}: webview label mismatch"
        ));
    }

    let app_webview = get_webview_in_sage_window(app_handle, webview_label)?;

    let current_url = app_webview
        .url()
        .map_err(|e| format!("failed to read current webview url: {e}"))?;

    if !is_allowed_app_url(&current_url, &app) {
        return Err(format!(
            "bridge denied for {webview_label}: current url {} is outside {}://{}/...",
            current_url,
            protocol_scheme_for_app(&app),
            app.origin_id()
        ));
    }

    Ok(BridgeOrigin { app, impostor_runtime: impostor_runtime.cloned() })
}

fn assert_bridge_version(request: &RustBridgeRequest) -> Result<(), RustBridgeInvokeResult> {
    if let Some(version) = &request.bridge_version && version != "v1" {
        return Err(RustBridgeInvokeResult::Immediate { response: RustBridgeResponse::error(
            &request.id,
            "unsupported_bridge_version",
            format!("Unsupported Sage bridge version: {version}"),
        ) });
    }

    Ok(())
}
