use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::runtime::stop::{kill_runtime, SystemKillRuntimeError};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerCloseSelf;

#[async_trait]
impl BridgeMethod for RuntimeManagerCloseSelf {
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

        match kill_runtime(
            tools.app_handle,
            tools.host_state,
            &app_id,
            "self_close",
        )
            .await
        {
            Ok(_) | Err(SystemKillRuntimeError::NotFound) => {
                Ok(Box::new(()))
            }
        }
    }
}
