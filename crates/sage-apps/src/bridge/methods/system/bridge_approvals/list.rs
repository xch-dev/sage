use async_trait::async_trait;
use serde::Serialize;
use specta::Type;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::state::list_pending_approvals;
use crate::bridge::types::PendingBridgeApproval;
use crate::capabilities::list::SystemBridgeCapability;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BridgeApprovalsListPending;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingBridgeApprovalView {
    pub approval_id: String,
    pub app_id: String,
    pub approval: RustBridgeApprovalRequest,
}

#[async_trait]
impl BridgeMethod for BridgeApprovalsListPending {
    fn name(&self) -> &'static str {
        "bridgeApprovals.listPending"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::BridgeApprovalList)
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
        let approvals = list_pending_approvals(tools.host_state)
            .await
            .into_iter()
            .map(PendingBridgeApprovalView::from)
            .collect::<Vec<_>>();
        Ok(Box::new(approvals))
    }
}

impl From<PendingBridgeApproval> for PendingBridgeApprovalView {
    fn from(approval: PendingBridgeApproval) -> Self {
        Self {
            approval_id: approval.approval_id,
            app_id: approval.app_id,
            approval: approval.approval,
        }
    }
}
