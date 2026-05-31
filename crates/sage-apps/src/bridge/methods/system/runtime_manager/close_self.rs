use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::SystemBridgeCapability;
use crate::runtime::{SystemKillRuntimeError, kill_runtime};

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

        match kill_runtime(tools.app_handle, tools.host_state, &app_id, "self_close").await {
            Ok(())
            | Err(SystemKillRuntimeError::NotFound | SystemKillRuntimeError::RuntimeSync(_)) => {
                Ok(Box::new(()))
            }
        }
    }
}
