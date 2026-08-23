use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools,
    RustBridgeRequest, UserBridgeCapability, current_scoped_key, parse_optional_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetKey;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletGetKeyParams {}

#[async_trait]
impl BridgeMethod for WalletGetKey {
    fn name(&self) -> &'static str {
        "wallet.getKey"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetKey)
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let _params: WalletGetKeyParams = parse_optional_params(self, request)?;

        let sage = tools.app_state.lock().await;
        let (result, _) = current_scoped_key(&ctx, &sage)?;

        Ok(Box::new(result))
    }
}
