use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools,
    RustBridgeRequest, UserBridgeCapability, current_scoped_wallet, parse_optional_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetWallet;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletGetWalletParams {}

#[async_trait]
impl BridgeMethod for WalletGetWallet {
    fn name(&self) -> &'static str {
        "wallet.getWallet"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetWallet)
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let _params: WalletGetWalletParams = parse_optional_params(self, request)?;

        let sage = tools.app_state.lock().await;
        let (result, _) = current_scoped_wallet(&ctx, &sage)?;

        Ok(Box::new(result))
    }
}
