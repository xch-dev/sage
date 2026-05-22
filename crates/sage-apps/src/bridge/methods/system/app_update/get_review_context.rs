use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::update::check_app_update_inner;
use crate::runtime::resolve_app;
use crate::types::{SageApp, SageAppUrlPreview, UserSageAppView};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateGetReviewContextParams {
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateReviewContext {
    pub app: UserSageAppView,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<SageAppUrlPreview>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppUpdateGetReviewContext;

#[async_trait]
impl BridgeMethod for AppUpdateGetReviewContext {
    fn name(&self) -> &'static str {
        "appUpdate.getReviewContext"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppUpdateRead)
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
        let params: AppUpdateGetReviewContextParams = parse_required_params(self, request)?;

        let preview = check_app_update_inner(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to check update for {}: {err}",
                    params.app_id
                ))
            })?;

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

        Ok(Box::new(AppUpdateReviewContext { app, preview }))
    }
}
