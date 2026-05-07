use async_trait::async_trait;
use serde::Serialize;
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use sage_api::GetKeys;

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

        let keys = sage.get_keys(GetKeys {}).map_err(|err| {
            BridgeMethodHandleError::internal_error(format!(
                "{} failed: {err}",
                self.name()
            ))
        })?;

        Ok(Box::new(WalletListWalletsResult {
            wallets: keys
                .keys
                .into_iter()
                .map(|key| SystemWalletView {
                    fingerprint: key.fingerprint,
                    name: key.name,
                    emoji: key.emoji,
                })
                .collect(),
        }))
    }
}
