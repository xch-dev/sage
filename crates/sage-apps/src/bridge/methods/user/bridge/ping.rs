use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandler, BridgeTools, RustBridgeRequest, bridge_result,
};

#[derive(Debug, Clone, Copy)]
pub struct BridgePing;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BridgePingResult {
    pub ok: bool,
    pub app_id: String,
    pub app_name: String,
}

impl BridgeMethodHandler for BridgePing {
    fn name(&self) -> &'static str {
        "bridge.ping"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::ungated()
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
        _tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        bridge_result(BridgePingResult {
            ok: true,
            app_id: ctx.app.id(),
            app_name: ctx.app.name(),
        })
    }
}
