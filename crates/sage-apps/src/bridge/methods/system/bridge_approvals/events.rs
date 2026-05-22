use crate::bridge::event_emit::SystemRuntimeEvent;
use crate::bridge::methods::system::PendingBridgeApprovalView;
use crate::bridge::types::PendingBridgeApproval;
use crate::capabilities::list::SystemBridgeCapability;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BridgeApprovalsChangedEvent {
    approvals: Vec<PendingBridgeApprovalView>,
}

impl BridgeApprovalsChangedEvent {
    pub(crate) fn new_from_list(approvals: Vec<PendingBridgeApproval>) -> Self {
        Self {
            approvals: approvals
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl SystemRuntimeEvent for BridgeApprovalsChangedEvent {
    const TYPE: &'static str = "bridgeApproval.changed";
    const REQUIRED_CAPABILITY: SystemBridgeCapability =
        SystemBridgeCapability::BridgeApprovalListenApprovalsChanged;
}
