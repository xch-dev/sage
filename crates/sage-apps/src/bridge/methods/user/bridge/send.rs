use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::capabilities::list::UserBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::runtime::SageAppRuntimeImpostorKind;

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
        check_ctx(&ctx)?;

        let payload: BridgeSendRequest = parse_required_params(self, request)?;

        let payload_value = serde_json::to_value(&payload).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to encode bridge.send payload: {err}"
            ))
        })?;

        crate::sandbox::ingest_bridge_send_payload(&ctx.app.id(), &payload_value, tools.host_state)
            .await;

        Ok(Box::new(BridgeSendResult { ok: true }))
    }
}

fn check_ctx(ctx: &BridgeContext<'_>) -> Result<(), BridgeMethodHandleError> {
    let is_sandbox_test = ctx.app.with(|app| app.common().is_sandbox_test());

    let is_storage_clear_probe = ctx
        .impostor_runtime
        .as_ref()
        .is_some_and(|runtime| {
            runtime.kind() == SageAppRuntimeImpostorKind::StorageClearProbe
        });

    if !is_sandbox_test && !is_storage_clear_probe {
        return Err(BridgeMethodHandleError::invalid_request(
            "Method use is not allowed",
        ));
    }

    Ok(())
}
