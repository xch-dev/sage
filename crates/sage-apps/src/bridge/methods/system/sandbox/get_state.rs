use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::sandbox::state_view::build_state_view;

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
