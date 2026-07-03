use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest, SageAppView,
    SageGrantedPermissionsInput, SystemBridgeCapability, apply_app_update_inner,
    parse_required_params,
};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateApplyUpdateParams {
    pub app_id: String,
    pub additional_granted_permissions: SageGrantedPermissionsInput,
    /// Manifest hash of the pending update the user actually reviewed. Apply is
    /// rejected if the pending update no longer matches this hash.
    pub reviewed_manifest_hash: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateApplyUpdateResult {
    pub app: SageAppView,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppUpdateApplyUpdate;

#[async_trait]
impl BridgeMethod for AppUpdateApplyUpdate {
    fn name(&self) -> &'static str {
        "appUpdate.applyUpdate"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppUpdateApply)
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
        let params: AppUpdateApplyUpdateParams = parse_required_params(self, request)?;

        let app = apply_app_update_inner(
            tools.app_handle,
            tools.host_state,
            &params.app_id,
            Some(params.additional_granted_permissions),
            Some(params.reviewed_manifest_hash),
        )
        .await
        .map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to apply update for {}: {err}",
                params.app_id
            ))
        })?;

        Ok(Box::new(AppUpdateApplyUpdateResult { app }))
    }
}
