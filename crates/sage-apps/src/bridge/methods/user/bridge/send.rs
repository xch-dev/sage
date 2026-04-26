use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};
use crate::bridge::{RustBridgeRequest};

#[derive(Debug, Clone, Copy)]
pub struct BridgeSend;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeSendRequest {
    pub kind: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BridgeSendResult {
    pub ok: bool,
}

#[async_trait]
impl BridgeMethod for BridgeSend {
    fn name(&self) -> &'static str {
        "bridge.send"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::BridgeSend)
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
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let payload: BridgeSendRequest = parse_required_params(self, request)?;

        let payload_value = serde_json::to_value(&payload).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to encode bridge.send payload: {err}"
            ))
        })?;

        crate::sandbox::ingest_bridge_send_payload(
            &ctx.app.id(),
            &payload_value,
            tools.host_state,
        )
            .await;

        Ok(Box::new(BridgeSendResult { ok: true }))
    }
}
