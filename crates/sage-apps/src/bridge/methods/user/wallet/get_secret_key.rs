use async_trait::async_trait;
use sage_api::GetSecretKey;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::types::RustBridgeApprovalBody;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetSecretKey;

#[async_trait]
impl BridgeMethod for WalletGetSecretKey {
    fn name(&self) -> &'static str {
        "wallet.getSecretKey"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetSecretKey)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: GetSecretKey = parse_required_params(self, request)?;

        Ok(Some(RustBridgeApprovalRequest {
            app: ctx.app.into(),
            source_label: ctx.app.webview_label(),
            request_id: request.id.clone(),
            body: RustBridgeApprovalBody::GetSecretKey {
                fingerprint: params.fingerprint,
            },
        }))
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetSecretKey = parse_required_params(self, request)?;

        let sage = tools.app_state.lock().await;

        let response = sage.get_secret_key(params).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
        })?;

        Ok(Box::new(response))
    }
}
