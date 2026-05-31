use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::bridge::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};

#[derive(Debug, Clone, Copy)]
pub struct BridgePing;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BridgePingResult {
    pub ok: bool,
    pub app_id: String,
    pub app_name: String,
}

#[async_trait]
impl BridgeMethod for BridgePing {
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
        Ok(Box::new(BridgePingResult {
            ok: true,
            app_id: ctx.app.id(),
            app_name: ctx.app.name(),
        }))
    }
}
