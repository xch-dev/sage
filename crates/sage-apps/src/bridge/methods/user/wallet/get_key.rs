use async_trait::async_trait;
use sage_api::GetKey;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::user::wallet::require_scoped_fingerprint;
use crate::capabilities::list::UserBridgeCapability;

#[derive(Debug, Clone, Copy)]
pub struct WalletGetKey;

#[async_trait]
impl BridgeMethod for WalletGetKey {
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

        Ok(Box::new(result))
    }
}
