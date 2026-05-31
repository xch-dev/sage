use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandler, BridgeTools, RustBridgeRequest, SystemBridgeCapability,
    SystemKillRuntimeError, bridge_result, kill_runtime,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerCloseSelf;

impl BridgeMethodHandler for RuntimeManagerCloseSelf {
    fn name(&self) -> &'static str {
        "runtimeManager.closeSelf"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerCloseSelf)
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
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let app_id = ctx.app.id();

        match kill_runtime(tools.app_handle, tools.host_state, &app_id, "self_close").await {
            Ok(())
            | Err(SystemKillRuntimeError::NotFound | SystemKillRuntimeError::RuntimeSync(_)) => {
                bridge_result(())
            }
        }
    }
}
