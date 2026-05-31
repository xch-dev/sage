use async_trait::async_trait;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools, RustBridgeRequest, SageApp, SharedCapabilitiesExt, UserBridgeCapability};

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
        let effective_capabilities = ctx.app.with(|app| match app {
            SageApp::User(user_app) => user_app
                .common()
                .requested_permissions()
                .capabilities()
                .resolve_effective_grants(
                    user_app
                        .common()
                        .granted_permissions()
                        .capabilities()
                        .copied(),
                ),

            SageApp::System(_) => app.granted_permissions().capabilities().copied().collect(),
        });

        let shared_capabilities = effective_capabilities.shared();

        Ok(Box::new(shared_capabilities))
    }
}
