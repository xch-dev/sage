use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult,
    BridgeMethodCapability, BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::install::commands::preview_app_url;

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallPreviewUrlParams {
    app_url: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallPreviewUrl;

#[async_trait]
impl BridgeMethod for AppInstallPreviewUrl {
    fn name(&self) -> &'static str {
        "appInstall.previewUrl"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppInstallPreview)
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
        _tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: AppInstallPreviewUrlParams = parse_required_params(self, request)?;

        let preview = preview_app_url(params.app_url)
            .await
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(preview))
    }
}
