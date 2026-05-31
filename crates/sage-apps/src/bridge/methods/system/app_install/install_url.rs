use std::io;

use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::AppsHostState;
use crate::bridge::AppInstallInstallResult;
use crate::bridge::RustBridgeRequest;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::host::AppState;
use crate::lifecycle::install_app_from_source;
use crate::types::{SageAppUrl, SageAppWalletScope, SageGrantedPermissionsInput, UserSageAppView};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallInstallUrlParams {
    app_url: String,
    granted_permissions: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
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
            tools.host_state.clone(),
            params.app_url,
            params.granted_permissions,
            params.wallet_scope,
        )
        .await
        .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(AppInstallInstallResult::new(app)))
    }
}

pub async fn install_app_url(
    app: AppHandle,
    state: State<'_, AppState>,
    host_state: State<'_, AppsHostState>,
    app_url: String,
    granted_permissions_input: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
) -> crate::host::Result<UserSageAppView> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };
    let parsed_app_url = SageAppUrl::parse(&app_url)
        .map_err(|err| io::Error::other(format!("invalid app URL {app_url}: {err}")))?;
    let result = install_app_from_source(
        &app,
        &host_state,
        &base_path,
        granted_permissions_input,
        wallet_scope,
        parsed_app_url,
    )
    .await;

    result.map(|app| (&app).into()).map_err(|err| {
        io::Error::other(format!("failed to install app URL {app_url}: {err}")).into()
    })
}
