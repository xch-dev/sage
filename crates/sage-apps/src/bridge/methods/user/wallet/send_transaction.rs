use async_trait::async_trait;
use sage_api::Amount;
use sage_api::wallet_connect::{
    Coin, CoinSpend, SendTransactionImmediately, SendTransactionImmediatelyResponse, SpendBundle,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletSendTransaction;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(untagged)]
pub enum WalletSendTransactionAmount {
    String(String),
    Number(u64),
}

impl From<WalletSendTransactionAmount> for Amount {
    fn from(amount: WalletSendTransactionAmount) -> Self {
        match amount {
            WalletSendTransactionAmount::String(value) => Self::String(value),
            WalletSendTransactionAmount::Number(value) => Self::Number(value),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletSendTransactionCoin {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: WalletSendTransactionAmount,
}

impl From<WalletSendTransactionCoin> for Coin {
    fn from(coin: WalletSendTransactionCoin) -> Self {
        Self {
            parent_coin_info: coin.parent_coin_info,
            puzzle_hash: coin.puzzle_hash,
            amount: coin.amount.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletSendTransactionCoinSpend {
    pub coin: WalletSendTransactionCoin,
    pub puzzle_reveal: String,
    pub solution: String,
}

impl From<WalletSendTransactionCoinSpend> for CoinSpend {
    fn from(spend: WalletSendTransactionCoinSpend) -> Self {
        Self {
            coin: spend.coin.into(),
            puzzle_reveal: spend.puzzle_reveal,
            solution: spend.solution,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletSendTransactionSpendBundle {
    pub coin_spends: Vec<WalletSendTransactionCoinSpend>,
    pub aggregated_signature: String,
}

impl From<WalletSendTransactionSpendBundle> for SpendBundle {
    fn from(bundle: WalletSendTransactionSpendBundle) -> Self {
        Self {
            coin_spends: bundle.coin_spends.into_iter().map(Into::into).collect(),
            aggregated_signature: bundle.aggregated_signature,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletSendTransactionParams {
    pub spend_bundle: WalletSendTransactionSpendBundle,
}

impl From<WalletSendTransactionParams> for SendTransactionImmediately {
    fn from(params: WalletSendTransactionParams) -> Self {
        Self {
            spend_bundle: params.spend_bundle.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSendTransactionResult {
    pub status: u8,
    pub error: Option<String>,
}

fn map_response(
    response: SendTransactionImmediatelyResponse,
) -> Result<WalletSendTransactionResult, BridgeMethodHandleError> {
    if !matches!(response.status, 1..=3) {
        return Err(BridgeMethodHandleError::internal_error(format!(
            "wallet returned unsupported mempool status: {}",
            response.status
        )));
    }

    Ok(WalletSendTransactionResult {
        status: response.status,
        error: response.error,
    })
}

fn validate_params(params: &WalletSendTransactionParams) -> Result<(), BridgeMethodHandleError> {
    if params.spend_bundle.coin_spends.is_empty() {
        return Err(BridgeMethodHandleError::invalid_request(
            "spendBundle.coin_spends must contain at least one coin spend",
        ));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for WalletSendTransaction {
    fn name(&self) -> &'static str {
        "wallet.sendTransaction"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletSendTransaction)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: WalletSendTransactionParams = parse_required_params(self, request)?;
        validate_params(&params)?;
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletSendTransactionParams = parse_required_params(self, request)?;
        validate_params(&params)?;

        let response = tools
            .app_state
            .lock()
            .await
            .send_transaction_immediately(params.into())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(map_response(response)?))
    }
}
