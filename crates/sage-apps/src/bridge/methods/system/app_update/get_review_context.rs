use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, CorruptedInstalledSageApp,
    RustBridgeRequest, SageApp, SageAppCompatibility, SageAppUrlPreview, SystemBridgeCapability,
    UserSageAppPendingUpdate, UserSageAppPendingUpdateView, UserSageAppView,
    check_app_update_inner, parse_required_params, resolve_app,
};

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateGetReviewContextParams {
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateReviewContext {
    pub target: AppUpdateTargetView,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<SageAppUrlPreview>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<SageAppCompatibility>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AppUpdateTargetView {
    Installed {
        app: Box<UserSageAppView>,
    },
    Recoverable {
        app: Box<CorruptedInstalledSageApp>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "pendingUpdate")]
        pending_update: Option<Box<UserSageAppPendingUpdateView>>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppUpdateGetReviewContext;

#[async_trait]
impl BridgeMethod for AppUpdateGetReviewContext {
    fn name(&self) -> &'static str {
        "appUpdate.getReviewContext"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppUpdateRead)
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
        let params: AppUpdateGetReviewContextParams = parse_required_params(self, request)?;

        let preview = check_app_update_inner(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to check update for {}: {err}",
                    params.app_id
                ))
            })?;

        let target = match resolve_app(tools.app_handle, &params.app_id).await {
            Ok(resolved) => {
                let app = resolved
                    .with_app(|app| {
                        app.with(|sage_app| match sage_app {
                            SageApp::User(user_app) => Some(UserSageAppView::from(user_app)),
                            SageApp::System(_) => None,
                        })
                    })
                    .ok_or_else(|| {
                        BridgeMethodHandleError::invalid_request(format!(
                            "app {} is not a user app",
                            params.app_id
                        ))
                    })?;

                AppUpdateTargetView::Installed { app: Box::new(app) }
            }
            Err(resolve_error) => {
                let recoverable = tools
                    .host_state
                    .db
                    .get_corrupted_user_app(
                        &params.app_id,
                        anyhow::anyhow!(resolve_error.to_string()),
                    )
                    .await
                    .map_err(|err| {
                        BridgeMethodHandleError::internal_error(format!(
                            "failed to load recovery state for {}: {err}",
                            params.app_id
                        ))
                    })?;

                let recoverable = recoverable
                    .with_evaluated_compatibility(&tools.app_handle.package_info().version);

                let recovery = tools
                    .host_state
                    .db
                    .get_recoverable_user_app(&params.app_id)
                    .await
                    .map_err(|err| {
                        BridgeMethodHandleError::internal_error(format!(
                            "failed to load durable recovery state for {}: {err}",
                            params.app_id
                        ))
                    })?
                    .ok_or_else(|| {
                        BridgeMethodHandleError::internal_error(format!(
                            "recovery state disappeared for {}",
                            params.app_id
                        ))
                    })?;

                let pending_update =
                    preview
                        .as_ref()
                        .and_then(recovery_pending_update)
                        .map(|pending| {
                            UserSageAppPendingUpdateView::from_pending_update(
                                &pending,
                                recovery.granted_permissions(),
                            )
                        });

                AppUpdateTargetView::Recoverable {
                    app: Box::new(recoverable),
                    pending_update: pending_update.map(Box::new),
                }
            }
        };

        let compatibility = preview.as_ref().map(|preview| {
            SageAppCompatibility::for_app(
                tools.app_handle,
                &preview.manifest().manifest_header().sage_version,
            )
        });

        Ok(Box::new(AppUpdateReviewContext {
            target,
            preview,
            compatibility,
        }))
    }
}

fn recovery_pending_update(preview: &SageAppUrlPreview) -> Option<UserSageAppPendingUpdate> {
    Some(UserSageAppPendingUpdate::new(
        preview.app_url().clone(),
        preview.manifest_hash().to_string(),
        preview.full_manifest()?.clone(),
    ))
}
