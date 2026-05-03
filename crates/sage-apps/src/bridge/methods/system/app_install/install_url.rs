use async_trait::async_trait;
use serde::{Deserialize};
use specta::Type;
use tauri::Manager;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult,
    BridgeMethodCapability, BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::system::AppInstallInstallResult;
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::install::commands::install_app_url;
use crate::types::{SageGrantedPermissionsInput};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallInstallUrlParams {
    app_url: String,
    granted_permissions: SageGrantedPermissionsInput,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallInstallUrl;

#[async_trait]
impl BridgeMethod for AppInstallInstallUrl {
    fn name(&self) -> &'static str {
        "appInstall.installUrl"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppInstallApply)
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
        let params: AppInstallInstallUrlParams = parse_required_params(self, request)?;
        let state = tools.app_handle.state::<crate::host::AppState>();

        let app = install_app_url(
            tools.app_handle.clone(),
            state,
            params.app_url,
            params.granted_permissions,
        )
            .await
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(AppInstallInstallResult::new(app)))
    }
}
