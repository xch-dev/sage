use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::runtime::{SageAppRuntimeRecordView, list_runtimes};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerListRuntimes;

#[async_trait]
impl BridgeMethod for RuntimeManagerListRuntimes {
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

        Ok(Box::new(runtime_views))
    }
}
