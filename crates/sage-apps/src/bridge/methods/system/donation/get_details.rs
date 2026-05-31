use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeRequest, SageAppIconView,
    SystemBridgeCapability, bridge_result, parse_required_params, resolve_app,
};

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DonationGetDetailsParams {
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DonationDetails {
    pub app_id: String,
    pub app_name: String,
    pub app_icon: Option<SageAppIconView>,
    pub author_name: Option<String>,
    pub author_avatar: Option<SageAppIconView>,
    pub donation_address: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DonationGetDetails;

impl BridgeMethodHandler for DonationGetDetails {
    fn name(&self) -> &'static str {
        "donations.getDetails"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::DonationGetDetails)
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
        let params: DonationGetDetailsParams = parse_required_params(self, request)?;

        let resolved = resolve_app(tools.app_handle, &params.app_id)
            .await
            .map_err(|err| BridgeMethodHandleError::invalid_request(err.to_string()))?;

        let details = resolved
            .with_app(|app| {
                app.with(|app| {
                    let common = app.common();
                    let manifest = common.active_snapshot().manifest();

                    let donation = manifest
                        .donation()
                        .ok_or_else(|| "App does not have a donation address".to_string())?;

                    let author = manifest.author();

                    Ok::<DonationDetails, String>(DonationDetails {
                        app_id: app.id().to_string(),
                        app_name: manifest.name().to_string(),
                        app_icon: SageAppIconView::from_common(common),
                        author_name: author.map(|author| author.name().to_string()),
                        author_avatar: SageAppIconView::author_avatar_from_common(common),
                        donation_address: donation.address().to_string(),
                    })
                })
            })
            .map_err(BridgeMethodHandleError::invalid_request)?;

        bridge_result(details)
    }
}
