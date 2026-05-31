use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::runtime::{RuntimeTargetParams, SageAppRuntimeRecordView, hide_runtime};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerHideRuntime;

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

        let runtime = hide_runtime(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        let runtime_view: SageAppRuntimeRecordView = runtime.into();
        Ok(Box::new(runtime_view))
    }
}
