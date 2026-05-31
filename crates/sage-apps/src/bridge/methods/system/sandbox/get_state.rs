use async_trait::async_trait;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools, build_state_view, RustBridgeRequest, SystemBridgeCapability};

#[derive(Debug, Clone, Copy)]
pub struct SandboxGetState;

#[async_trait]
impl BridgeMethod for SandboxGetState {
    fn name(&self) -> &'static str {
        "sandbox.getState"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::SandboxGetState)
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
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let state = build_state_view(tools.host_state).await;

        Ok(Box::new(state))
    }
}
