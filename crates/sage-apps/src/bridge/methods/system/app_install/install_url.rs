use std::io;
use async_trait::async_trait;
use serde::{Deserialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult,
    BridgeMethodCapability, BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::system::AppInstallInstallResult;
use crate::capabilities::list::SystemBridgeCapability;
use crate::host::AppState;
use crate::lifecycle::install::install_app_from_source;
use crate::types::{SageAppUrl, SageGrantedPermissionsInput, UserSageAppView};

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
        let state = tools.app_handle.state::<AppState>();

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

pub async fn install_app_url(
    app: AppHandle,
    state: State<'_, AppState>,
    app_url: String,
    granted_permissions_input: SageGrantedPermissionsInput,
) -> crate::host::Result<UserSageAppView> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };
    let parsed_app_url = SageAppUrl::parse(&app_url)
        .map_err(|err| io::Error::other(format!("invalid app URL {app_url}: {err}")))?;
    let result = install_app_from_source(&app, &base_path, granted_permissions_input, parsed_app_url)
        .await;

    result
        .map(|app| (&app).into())
        .map_err(|err| {
            io::Error::other(format!("failed to install app URL {app_url}: {err}")).into()
        })
}
