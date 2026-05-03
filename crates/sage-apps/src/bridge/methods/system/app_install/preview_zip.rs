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
use crate::lifecycle::install::commands::preview_app_zip;

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallPreviewZipParams {
    zip_path: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallPreviewZip;

#[async_trait]
impl BridgeMethod for AppInstallPreviewZip {
    fn name(&self) -> &'static str {
        "appInstall.previewZip"
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
        let params: AppInstallPreviewZipParams = parse_required_params(self, request)?;

        let manifest = preview_app_zip(params.zip_path)
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(manifest))
    }
}
