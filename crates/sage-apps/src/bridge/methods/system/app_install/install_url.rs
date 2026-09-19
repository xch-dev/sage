use std::{io, sync::Arc};

use async_trait::async_trait;
use serde::Deserialize;
use specta::Type;
use tauri::{AppHandle, Manager, State};

use crate::{
    AppInstallDownloadProgressEvent, AppInstallInstallResult, AppState, AppsHostState,
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools,
    ProgressReportingUrlInstallSource, Result, RustBridgeRequest, SageAppUrl, SageAppWalletScope,
    SageGrantedPermissionsInput, SnapshotProgressReporter, SystemBridgeCapability, UserSageAppView,
    emit_system_runtime_event_to_app, install_app_from_source, parse_required_params,
};

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
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: AppInstallInstallUrlParams = parse_required_params(self, request)?;
        let state = tools.app_handle.state::<AppState>();
        let progress_app_handle = tools.app_handle.clone();
        let progress_app = ctx.app.clone();
        let progress_reporter: SnapshotProgressReporter = Arc::new(move |progress| {
            let _ = emit_system_runtime_event_to_app(
                &progress_app_handle,
                &progress_app,
                AppInstallDownloadProgressEvent {
                    downloaded_bytes: progress.downloaded_bytes,
                    total_bytes: progress.total_bytes,
                },
            );
        });

        let app = install_app_url(
            tools.app_handle.clone(),
            state,
            tools.host_state.clone(),
            params.app_url,
            params.granted_permissions,
            params.wallet_scope,
            progress_reporter,
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
    progress_reporter: SnapshotProgressReporter,
) -> Result<UserSageAppView> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };
    let parsed_app_url = SageAppUrl::parse(&app_url)
        .map_err(|err| io::Error::other(format!("invalid app URL {app_url}: {err}")))?;
    let source = ProgressReportingUrlInstallSource::new(parsed_app_url, progress_reporter);
    let result = install_app_from_source(
        &app,
        &host_state,
        &base_path,
        granted_permissions_input,
        wallet_scope,
        source,
    )
    .await;

    result.map(|app| (&app).into()).map_err(|err| {
        io::Error::other(format!("failed to install app URL {app_url}: {err}")).into()
    })
}
