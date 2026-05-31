use sage_api::GetSecretKey;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, bridge_result,
    parse_required_params, require_scoped_fingerprint,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetSecretKey;

impl BridgeMethodHandler for WalletGetSecretKey {
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

        bridge_result(response)
    }
}
