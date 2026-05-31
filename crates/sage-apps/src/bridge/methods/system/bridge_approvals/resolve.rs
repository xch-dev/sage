use async_trait::async_trait;

use crate::bridge::process_after_approval;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::{ResolveBridgeApprovalArgs, RustBridgeRequest};
use crate::capabilities::SystemBridgeCapability;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BridgeApprovalsResolve;

#[async_trait]
impl BridgeMethod for BridgeApprovalsResolve {
    fn name(&self) -> &'static str {
        "bridgeApprovals.resolve"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::BridgeApprovalResolve)
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
        let params: ResolveBridgeApprovalArgs = parse_required_params(self, request)?;

        process_after_approval(tools.app_handle, tools.app_state, tools.host_state, params)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        Ok(Box::new(()))
    }
}
