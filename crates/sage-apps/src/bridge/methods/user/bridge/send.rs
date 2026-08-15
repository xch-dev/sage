use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID, BridgeApprovalRequestResult, BridgeContext,
    BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeMethodHandleError, BridgeTools,
    RustBridgeRequest, UserBridgeCapability, ingest_bridge_send_payload,
    ingest_origin_cleanup_bridge_send_payload, parse_required_params,
};

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

#[derive(Debug, Clone, Copy)]
enum BridgeSendContextKind {
    Sandbox,
    OriginCleanup,
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
        ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        check_ctx(&ctx)?;
        Ok(None)
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let ctx_kind = check_ctx(&ctx)?;

        let payload: BridgeSendRequest = parse_required_params(self, request)?;

        let payload_value = serde_json::to_value(&payload).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to encode bridge.send payload: {err}"
            ))
        })?;

        match ctx_kind {
            BridgeSendContextKind::Sandbox => {
                ingest_bridge_send_payload(&ctx.app.id(), &payload_value, tools.host_state).await;
            }

            BridgeSendContextKind::OriginCleanup => {
                ingest_origin_cleanup_bridge_send_payload(
                    &ctx.app.id(),
                    &payload_value,
                    tools.host_state,
                )
                .await
                .map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "failed to ingest origin cleanup payload: {err}"
                    ))
                })?;
            }
        }

        Ok(Box::new(BridgeSendResult { ok: true }))
    }
}

fn check_ctx(ctx: &BridgeContext<'_>) -> Result<BridgeSendContextKind, BridgeMethodHandleError> {
    let is_sandbox_test = ctx.app.with(|app| app.common().is_sandbox_test());
    if is_sandbox_test {
        return Ok(BridgeSendContextKind::Sandbox);
    }

    let app_id = ctx.app.id();
    if app_id == BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID {
        return Ok(BridgeSendContextKind::OriginCleanup);
    }

    Err(BridgeMethodHandleError::invalid_request(
        "Method use is not allowed",
    ))
}
