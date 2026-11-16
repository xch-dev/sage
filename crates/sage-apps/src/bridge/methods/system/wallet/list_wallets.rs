use async_trait::async_trait;
use sage_api::GetWallets;
use serde::Serialize;
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    SystemBridgeCapability,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct WalletListWallets;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemWalletView {
    pub fingerprint: u32,
    pub name: String,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletListWalletsResult {
    pub wallets: Vec<SystemWalletView>,
}

#[async_trait]
impl BridgeMethod for WalletListWallets {
    fn name(&self) -> &'static str {
        "wallet.listWallets"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::WalletListWallets)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let sage = tools.app_state.lock().await;

        let wallets = sage.get_wallets(GetWallets {}).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
        })?;

        Ok(Box::new(WalletListWalletsResult {
            wallets: wallets
                .wallets
                .into_iter()
                .map(|wallet| SystemWalletView {
                    fingerprint: wallet.fingerprint,
                    name: wallet.name,
                    emoji: wallet.emoji,
                })
                .collect(),
        }))
    }
}
