use async_trait::async_trait;
use sage_api::GetSecretKey;
use serde::Deserialize;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, current_scoped_key,
    parse_optional_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetSecretKey;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletGetSecretKeyParams {}

#[async_trait]
impl BridgeMethod for WalletGetSecretKey {
    fn name(&self) -> &'static str {
        "wallet.getSecretKey"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetSecretKey)
    }

    async fn prepare_approval(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let _params: WalletGetSecretKeyParams = parse_optional_params(self, request)?;
        let sage = tools.app_state.lock().await;
        let (_, fingerprint) = current_scoped_key(&ctx, &sage)?;

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::GetSecretKey { fingerprint },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let _params: WalletGetSecretKeyParams = parse_optional_params(self, request)?;

        let sage = tools.app_state.lock().await;
        let (_, fingerprint) = current_scoped_key(&ctx, &sage)?;

        let response = sage
            .get_secret_key(GetSecretKey { fingerprint })
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(response))
    }
}
