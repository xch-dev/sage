use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::methods::shared::{
    parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult,
    BridgeMethodCapability, BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::RustBridgeRequest;
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::update::commands::{apply_app_update, download_app_update};
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

        download_app_update(tools.app_handle.clone(), params.app_id.clone())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to download update for {}: {err}",
                    params.app_id
                ))
            })?;

        let app = apply_app_update(
            tools.app_state.clone(),
            tools.app_handle.clone(),
            params.app_id.clone(),
            params.granted_permissions,
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
