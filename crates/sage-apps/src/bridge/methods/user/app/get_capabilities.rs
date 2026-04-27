use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::capabilities::{SharedCapabilitiesExt, UserBridgeCapability};
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::types::SageApp;

#[derive(Debug, Clone, Copy)]
pub struct AppGetCapabilities;

#[async_trait]
impl BridgeMethod for AppGetCapabilities {
    fn name(&self) -> &'static str {
        "app.getCapabilities"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppGetCapabilities)
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
        _tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let effective_capabilities = match ctx.app {
            SageApp::User(user_app) => user_app
                .common
                .requested_permissions
                .capabilities
                .resolve_effective_grants(
                    user_app.common.granted_permissions.capabilities().copied(),
                )
                .unwrap_or_default(),

            SageApp::System(_) => ctx
                .app
                .granted_permissions()
                .capabilities()
                .copied()
                .collect(),
        };

        let shared_capabilities = effective_capabilities.shared();

        Ok(Box::new(shared_capabilities))
    }
}
