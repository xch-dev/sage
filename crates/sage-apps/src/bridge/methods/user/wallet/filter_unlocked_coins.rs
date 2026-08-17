use async_trait::async_trait;
use sage_api::wallet_connect::{FilterUnlockedCoins, FilterUnlockedCoinsResponse};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletFilterUnlockedCoins;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletFilterUnlockedCoinsParams {
    pub coin_names: Vec<String>,
}

impl From<WalletFilterUnlockedCoinsParams> for FilterUnlockedCoins {
    fn from(params: WalletFilterUnlockedCoinsParams) -> Self {
        Self {
            coin_ids: params.coin_names,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(transparent)]
pub struct WalletFilterUnlockedCoinsResult(pub Vec<String>);

impl From<FilterUnlockedCoinsResponse> for WalletFilterUnlockedCoinsResult {
    fn from(response: FilterUnlockedCoinsResponse) -> Self {
        Self(response.coin_ids)
    }
}

#[async_trait]
impl BridgeMethod for WalletFilterUnlockedCoins {
    fn name(&self) -> &'static str {
        "wallet.filterUnlockedCoins"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletFilterUnlockedCoins)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let _params: WalletFilterUnlockedCoinsParams = parse_required_params(self, request)?;
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletFilterUnlockedCoinsParams = parse_required_params(self, request)?;
        if params.coin_names.is_empty() {
            return Err(BridgeMethodHandleError::invalid_request(
                "coinNames must contain at least one coin ID",
            ));
        }

        let response = tools
            .app_state
            .lock()
            .await
            .filter_unlocked_coins(params.into())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletFilterUnlockedCoinsResult::from(response)))
    }
}
