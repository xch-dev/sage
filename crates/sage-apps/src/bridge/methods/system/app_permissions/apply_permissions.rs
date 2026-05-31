use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeRequest, SageApp,
    SageAppWalletScope, SageGrantedPermissionsInput, SystemBridgeCapability, UserSageAppView,
    bridge_result, parse_required_params, resolve_app, update_app_permissions_for_app,
    update_app_wallet_scope_for_app,
};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionsApplyPermissionsParams {
    pub app_id: String,
    pub granted_permissions: SageGrantedPermissionsInput,
    pub wallet_scope: SageAppWalletScope,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionsApplyPermissionsResult {
    pub app: UserSageAppView,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppPermissionsApplyPermissions;

impl BridgeMethodHandler for AppPermissionsApplyPermissions {
    fn name(&self) -> &'static str {
        "appPermissions.applyPermissions"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppPermissionsApply)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: AppPermissionsApplyPermissionsParams = parse_required_params(self, request)?;

        let resolved = resolve_app(tools.app_handle, &params.app_id)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to resolve app {}: {err}",
                    params.app_id
                ))
            })?;

        let requested =
            resolved.with_app(|app| app.with(|sage_app| sage_app.requested_permissions().clone()));

        let granted_permissions =
            params
                .granted_permissions
                .resolve(&requested)
                .map_err(|err| {
                    BridgeMethodHandleError::invalid_request(format!(
                        "invalid granted permissions: {err}"
                    ))
                })?;

        let app = resolved.clone_app_for_operation();

        update_app_permissions_for_app(
            tools.app_handle,
            tools.host_state,
            &app,
            &granted_permissions,
        )
        .await
        .map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to update app permissions: {err}"
            ))
        })?;
        update_app_wallet_scope_for_app(tools.app_handle, &app, params.wallet_scope)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to update app wallet scope: {err}"
                ))
            })?;

        let app_view = app
            .with(|sage_app| match sage_app {
                SageApp::User(user_app) => Some(UserSageAppView::from(user_app)),
                SageApp::System(_) => None,
            })
            .ok_or_else(|| {
                BridgeMethodHandleError::invalid_request(format!(
                    "app {} is not a user app",
                    params.app_id
                ))
            })?;

        bridge_result(AppPermissionsApplyPermissionsResult { app: app_view })
    }
}
