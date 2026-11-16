use async_trait::async_trait;
use sage_api::Amount;
use sage_api::wallet_connect::GetAssetCoins;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, WalletAssetCoinType, parse_required_params,
};

const BALANCE_PAGE_SIZE: u32 = 100;

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

fn amount_value(amount: &Amount) -> Result<u128, BridgeMethodHandleError> {
    amount.to_u128().ok_or_else(|| {
        BridgeMethodHandleError::internal_error(format!("wallet returned invalid amount: {amount}"))
    })
}

fn checked_add(total: u128, value: u128) -> Result<u128, BridgeMethodHandleError> {
    total.checked_add(value).ok_or_else(|| {
        BridgeMethodHandleError::internal_error("asset balance exceeds supported range")
    })
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

        let mut confirmed = 0_u128;
        let mut spendable = 0_u128;
        let mut spendable_coin_count = 0_u32;
        let mut offset = 0_u32;
        let sage = tools.app_state.lock().await;

        loop {
            let coins = sage
                .get_asset_coins(GetAssetCoins {
                    kind: params.kind.map(Into::into),
                    asset_id: params.asset_id.clone(),
                    included_locked: Some(true),
                    offset: Some(offset),
                    limit: Some(BALANCE_PAGE_SIZE),
                })
                .await
                .map_err(|err| {
                    BridgeMethodHandleError::internal_error(format!(
                        "{} failed: {err}",
                        self.name()
                    ))
                })?;

            let page_len = u32::try_from(coins.len()).map_err(|_| {
                BridgeMethodHandleError::internal_error("asset coin page is too large")
            })?;

            for record in coins {
                let amount = amount_value(&record.coin.amount)?;
                confirmed = checked_add(confirmed, amount)?;

                if !record.locked {
                    spendable = checked_add(spendable, amount)?;
                    spendable_coin_count =
                        spendable_coin_count.checked_add(1).ok_or_else(|| {
                            BridgeMethodHandleError::internal_error("spendable coin count overflow")
                        })?;
                }
            }

            if page_len < BALANCE_PAGE_SIZE {
                break;
            }

            offset = offset.checked_add(page_len).ok_or_else(|| {
                BridgeMethodHandleError::internal_error("asset coin pagination overflow")
            })?;
        }

        Ok(Box::new(WalletGetAssetBalanceResult {
            confirmed: confirmed.to_string(),
            spendable: spendable.to_string(),
            spendable_coin_count,
        }))
    }
}
