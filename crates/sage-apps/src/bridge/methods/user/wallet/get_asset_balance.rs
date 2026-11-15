use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, WalletAssetCoinType, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetAssetBalance;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletGetAssetBalanceParams {
    #[serde(default, rename = "type")]
    pub kind: Option<WalletAssetCoinType>,
    #[serde(default)]
    pub asset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletGetAssetBalanceResult {
    pub confirmed: String,
    pub spendable: String,
    pub spendable_coin_count: u32,
}

fn validate_params(params: &WalletGetAssetBalanceParams) -> Result<(), BridgeMethodHandleError> {
    if params.kind == Some(WalletAssetCoinType::Cat) && params.asset_id.is_none() {
        return Err(BridgeMethodHandleError::invalid_request(
            "assetId is required when type is cat",
        ));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for WalletGetAssetBalance {
    fn name(&self) -> &'static str {
        "wallet.getAssetBalance"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetAssetBalance)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: WalletGetAssetBalanceParams = parse_required_params(self, request)?;
        validate_params(&params)?;
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletGetAssetBalanceParams = parse_required_params(self, request)?;
        validate_params(&params)?;

        let (confirmed, spendable, spendable_coin_count) = tools
            .wallet()
            .await?
            .get_asset_balance(params.kind.map(Into::into), params.asset_id)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletGetAssetBalanceResult {
            confirmed,
            spendable,
            spendable_coin_count,
        }))
    }
}
