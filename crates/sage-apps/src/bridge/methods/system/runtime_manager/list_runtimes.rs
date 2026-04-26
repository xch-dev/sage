use async_trait::async_trait;

use crate::bridge::capabilities::SystemBridgeCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};
use crate::bridge::{RustBridgeRequest};
use crate::runtime::state::read::list_runtimes;

#[derive(Debug, Clone, Copy)]
pub struct RuntimeManagerListRuntimes;

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
        let records = list_runtimes(tools.host_state)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(records))
    }
}
