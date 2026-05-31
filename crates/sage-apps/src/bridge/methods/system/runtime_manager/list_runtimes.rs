use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeRequest,
    SageAppRuntimeRecordView, SystemBridgeCapability, bridge_result, list_runtimes,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerListRuntimes;

impl BridgeMethodHandler for RuntimeManagerListRuntimes {
    fn name(&self) -> &'static str {
        "runtimeManager.listRuntimes"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerListRuntimes)
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
        let runtimes = list_runtimes(tools.host_state)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        let runtime_views = runtimes
            .iter()
            .map(Into::into)
            .collect::<Vec<SageAppRuntimeRecordView>>();

        bridge_result(runtime_views)
    }
}
