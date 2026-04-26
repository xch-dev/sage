use async_trait::async_trait;

use crate::bridge::capabilities::SystemBridgeCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};
use crate::bridge::methods::system::runtime_manager::RuntimeTargetParams;
use crate::bridge::{RustBridgeRequest};
use crate::runtime::focus_runtime;

#[derive(Debug, Clone, Copy)]
pub struct RuntimeManagerFocusRuntime;

#[async_trait]
impl BridgeMethod for RuntimeManagerFocusRuntime {
    fn name(&self) -> &'static str {
        "runtimeManager.focusRuntime"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerFocusRuntime)
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

        let record = focus_runtime(
            tools.app_handle,
            tools.host_state,
            &params.app_id,
        )
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(record))
    }
}
