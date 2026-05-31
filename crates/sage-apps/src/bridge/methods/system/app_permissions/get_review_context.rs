use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, parse_required_params, resolve_app, RustBridgeRequest, SageApp, SystemBridgeCapability, UserSageAppView};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionsGetReviewContextParams {
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppPermissionsReviewContext {
    pub app: UserSageAppView,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppPermissionsGetReviewContext;

#[async_trait]
impl BridgeMethod for AppPermissionsGetReviewContext {
    fn name(&self) -> &'static str {
        "appPermissions.getReviewContext"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppPermissionsRead)
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
        let params: AppPermissionsGetReviewContextParams = parse_required_params(self, request)?;

        let resolved = resolve_app(tools.app_handle, &params.app_id)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to resolve app {}: {err}",
                    params.app_id
                ))
            })?;

        let app = resolved
            .with_app(|app| {
                app.with(|sage_app| match sage_app {
                    SageApp::User(user_app) => Some(UserSageAppView::from(user_app)),
                    SageApp::System(_) => None,
                })
            })
            .ok_or_else(|| {
                BridgeMethodHandleError::invalid_request(format!(
                    "app {} is not a user app",
                    params.app_id
                ))
            })?;

        Ok(Box::new(AppPermissionsReviewContext { app }))
    }
}
