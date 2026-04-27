use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::capabilities::SystemBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::system::runtime_manager::RuntimeTargetParams;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::runtime::hide_runtime;

#[derive(Debug, Clone, Copy)]
pub struct RuntimeManagerHideRuntime;

#[async_trait]
impl BridgeMethod for RuntimeManagerHideRuntime {
    fn name(&self) -> &'static str {
        "runtimeManager.hideRuntime"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerHideRuntime)
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
        let params: RuntimeTargetParams = parse_required_params(self, request)?;

        let record = hide_runtime(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(record))
    }
}
