use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RuntimeTargetParams,
    RustBridgeRequest, SageAppRuntimeRecordView, SystemBridgeCapability, bridge_result,
    focus_taskbar_runtime, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerFocusTaskbarRuntime;

impl BridgeMethodHandler for RuntimeManagerFocusTaskbarRuntime {
    fn name(&self) -> &'static str {
        "runtimeManager.focusTaskbarRuntime"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerFocusTaskbarRuntime)
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

        let runtime = focus_taskbar_runtime(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        let runtime_view: SageAppRuntimeRecordView = runtime.into();

        bridge_result(runtime_view)
    }
}
