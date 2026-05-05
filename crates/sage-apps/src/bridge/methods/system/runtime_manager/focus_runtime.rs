use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::capabilities::list::SystemBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::runtime::{focus_taskbar_runtime, RuntimeTargetParams, SageAppRuntimeRecordView};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerFocusRuntime;

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

        let runtime = focus_taskbar_runtime(tools.app_handle, tools.host_state, &params.app_id)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        let runtime_view: SageAppRuntimeRecordView = runtime.into();

        Ok(Box::new(runtime_view))
    }
}
