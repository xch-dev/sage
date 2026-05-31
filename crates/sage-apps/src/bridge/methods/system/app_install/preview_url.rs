use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::lifecycle::fetch_url_manifest_preview;
use crate::types::{SageAppUrl, SageAppUrlPreview};

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

        let preview = fetch_preview(params.app_url)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(preview))
    }
}

async fn fetch_preview(app_url: String) -> Result<SageAppUrlPreview, String> {
    let app_url =
        SageAppUrl::parse(&app_url).map_err(|err| format!("invalid app URL {app_url}: {err}"))?;

    let (manifest, manifest_hash) = fetch_url_manifest_preview(&app_url.manifest_url())
        .await
        .map_err(|err| format!("failed to fetch app manifest: {err}"))?;

    SageAppUrlPreview::new(&app_url, manifest, manifest_hash)
        .await
        .map_err(|err| format!("failed to preview app URL: {err}"))
}
