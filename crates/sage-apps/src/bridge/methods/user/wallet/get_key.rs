use sage_api::GetKey;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, bridge_result, parse_required_params, require_scoped_fingerprint,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetKey;

impl BridgeMethodHandler for WalletGetKey {
    fn name(&self) -> &'static str {
        "wallet.getKey"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetKey)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: GetKey = parse_required_params(self, request)?;

        require_scoped_fingerprint(&ctx, params.fingerprint)?;

        Ok(None)
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: GetKey = parse_required_params(self, request)?;

        require_scoped_fingerprint(&ctx, params.fingerprint)?;

        let sage = tools.app_state.lock().await;

        let result = sage.get_key(params).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "failed to execute {}: {err}",
                self.name()
            ))
        })?;

        bridge_result(result)
    }
}
