use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;
use tauri::Manager;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult,
    BridgeMethodCapability, BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::lifecycle::install::commands::install_app_zip;
use crate::types::SageGrantedPermissionsInput;

use super::AppInstallInstallResult;

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallInstallZipParams {
    zip_path: String,
    granted_permissions: SageGrantedPermissionsInput,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallInstallZip;

#[async_trait]
impl BridgeMethod for AppInstallInstallZip {
    fn name(&self) -> &'static str {
        "appInstall.installZip"
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
        let params: AppInstallInstallZipParams = parse_required_params(self, request)?;
        let state = tools.app_handle.state::<crate::host::AppState>();

        let app = install_app_zip(
            tools.app_handle.clone(),
            state,
            params.zip_path,
            params.granted_permissions,
        )
            .await
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(AppInstallInstallResult::new(app)))
    }
}
