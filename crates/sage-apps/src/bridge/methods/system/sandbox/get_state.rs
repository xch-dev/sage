use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandler, BridgeTools, RustBridgeRequest, SystemBridgeCapability, bridge_result,
    build_state_view,
};

#[derive(Debug, Clone, Copy)]
pub struct SandboxGetState;

impl BridgeMethodHandler for SandboxGetState {
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

        bridge_result(state)
    }
}
