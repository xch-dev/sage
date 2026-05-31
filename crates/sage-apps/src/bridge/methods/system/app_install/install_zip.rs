use std::{fs, io};

use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use super::AppInstallInstallResult;
use crate::{
    AppState, AppsHostState, BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult,
    BridgeMethod, BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, Result,
    RustBridgeRequest, SageAppWalletScope, SageGrantedPermissionsInput, SystemBridgeCapability,
    UserSageAppView, ZipInstallSource, apps_root, install_app_from_source, parse_required_params,
};

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
) -> Result<UserSageAppView> {
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
