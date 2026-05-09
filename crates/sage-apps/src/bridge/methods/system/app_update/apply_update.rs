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
use crate::lifecycle::update::logic::apply_app_update_inner;
use crate::types::{SageAppView, SageGrantedPermissionsInput};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateApplyUpdateParams {
    pub app_id: String,
    pub granted_permissions: SageGrantedPermissionsInput,
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
            Some(params.granted_permissions),
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
