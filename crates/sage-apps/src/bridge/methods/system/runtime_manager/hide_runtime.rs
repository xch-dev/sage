use async_trait::async_trait;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RuntimeTargetParams,
    RustBridgeRequest, SageAppRuntimeRecordView, SystemBridgeCapability, hide_runtime,
    parse_required_params,
};

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
