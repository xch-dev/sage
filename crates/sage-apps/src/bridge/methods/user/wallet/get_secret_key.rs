use async_trait::async_trait;
use sage_api::GetSecretKey;

use crate::bridge::RustBridgeApprovalBody;
use crate::bridge::require_scoped_fingerprint;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::capabilities::UserBridgeCapability;

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
        require_scoped_fingerprint(&ctx, Some(params.fingerprint))?;

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::GetSecretKey {
                fingerprint: params.fingerprint,
            },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetSecretKey = parse_required_params(self, request)?;
        require_scoped_fingerprint(&ctx, Some(params.fingerprint))?;

        let sage = tools.app_state.lock().await;

        let response = sage.get_secret_key(params).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
        })?;

        Ok(Box::new(response))
    }
}
