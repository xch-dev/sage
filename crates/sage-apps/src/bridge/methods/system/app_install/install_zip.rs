use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;
use std::{fs, io};
use tauri::{AppHandle, Manager, State};
use crate::AppsHostState;
use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::host::AppState;
use crate::lifecycle::apps_root;
use crate::lifecycle::install::install_app_from_source;
use crate::lifecycle::install::zip::ZipInstallSource;
use crate::types::{SageAppWalletScope, SageGrantedPermissionsInput, UserSageAppView};

use super::AppInstallInstallResult;

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallInstallZipParams {
    zip_path: String,
    granted_permissions: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
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
        let state = tools.app_handle.state::<AppState>();

        let app = install_app_zip(
            tools.app_handle.clone(),
            state,
            tools.host_state.clone(),
            params.zip_path,
            params.granted_permissions,
            params.wallet_scope,
        )
        .await
        .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;

        Ok(Box::new(AppInstallInstallResult::new(app)))
    }
}

pub async fn install_app_zip(
    app: AppHandle,
    state: State<'_, AppState>,
    host_state: State<'_, AppsHostState>,
    zip_path: String,
    granted_permissions_input: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
) -> crate::host::Result<UserSageAppView> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let root = apps_root(&base_path);

    fs::create_dir_all(&root).map_err(|err| {
        io::Error::other(format!(
            "failed to create apps directory {}: {err}",
            root.display()
        ))
    })?;

    let source = ZipInstallSource::new(&root, zip_path.clone());
    let unpack_dir = source.unpack_dir.clone();

    let result = install_app_from_source(
        &app,
        &host_state,
        &base_path,
        granted_permissions_input,
        wallet_scope,
        source,
    )
    .await;

    if unpack_dir.exists() {
        let _ = fs::remove_dir_all(&unpack_dir);
    }

    result.map(|app| (&app).into()).map_err(|err| {
        io::Error::other(format!("failed to install app zip {zip_path}: {err}")).into()
    })
}
