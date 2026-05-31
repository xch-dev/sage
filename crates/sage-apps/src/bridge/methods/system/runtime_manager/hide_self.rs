use async_trait::async_trait;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, hide_runtime, RustBridgeRequest, SystemBridgeCapability};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerHideSelf;

#[async_trait]
impl BridgeMethod for RuntimeManagerHideSelf {
    fn name(&self) -> &'static str {
        "runtimeManager.hideSelf"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerHideSelf)
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
        hide_runtime(tools.app_handle, tools.host_state, &ctx.app.id())
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(()))
    }
}
