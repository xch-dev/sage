use async_trait::async_trait;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, ResolveBridgeApprovalArgs,
    RustBridgeRequest, SystemBridgeCapability, parse_required_params, process_after_approval,
};

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
