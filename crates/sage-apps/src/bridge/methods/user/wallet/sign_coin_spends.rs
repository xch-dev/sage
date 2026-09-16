use async_trait::async_trait;
use sage_api::{
    Amount, Asset, AssetKind, CoinJson, CoinSpendJson, SignCoinSpends, SignCoinSpendsResponse,
    TransactionInput, TransactionOutput, TransactionSummary, ViewCoinSpends,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletSignCoinSpends;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(untagged)]
pub enum WalletSignCoinSpendsAmount {
    String(String),
    Number(u64),
}

impl From<WalletSignCoinSpendsAmount> for Amount {
    fn from(amount: WalletSignCoinSpendsAmount) -> Self {
        match amount {
            WalletSignCoinSpendsAmount::String(value) => Self::String(value),
            WalletSignCoinSpendsAmount::Number(value) => Self::Number(value),
        }
    }
}

impl From<Amount> for WalletSignCoinSpendsAmount {
    fn from(amount: Amount) -> Self {
        match amount {
            Amount::String(value) => Self::String(value),
            Amount::Number(value) => Self::Number(value),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletSignCoinSpendsCoin {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: WalletSignCoinSpendsAmount,
}

impl From<WalletSignCoinSpendsCoin> for CoinJson {
    fn from(coin: WalletSignCoinSpendsCoin) -> Self {
        Self {
            parent_coin_info: coin.parent_coin_info,
            puzzle_hash: coin.puzzle_hash,
            amount: coin.amount.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletSignCoinSpend {
    pub coin: WalletSignCoinSpendsCoin,
    pub puzzle_reveal: String,
    pub solution: String,
}

impl From<WalletSignCoinSpend> for CoinSpendJson {
    fn from(spend: WalletSignCoinSpend) -> Self {
        Self {
            coin: spend.coin.into(),
            puzzle_reveal: spend.puzzle_reveal,
            solution: spend.solution,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignCoinSpendsParams {
    pub coin_spends: Vec<WalletSignCoinSpend>,
    #[serde(default)]
    pub partial_sign: Option<bool>,
}

impl WalletSignCoinSpendsParams {
    fn into_request(self) -> SignCoinSpends {
        SignCoinSpends {
            coin_spends: self.coin_spends.into_iter().map(Into::into).collect(),
            auto_submit: false,
            partial: self.partial_sign.unwrap_or(false),
        }
    }

    fn preview_request(&self) -> ViewCoinSpends {
        ViewCoinSpends {
            coin_spends: self
                .coin_spends
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(transparent)]
pub struct WalletSignCoinSpendsResult(pub String);

impl From<SignCoinSpendsResponse> for WalletSignCoinSpendsResult {
    fn from(response: SignCoinSpendsResponse) -> Self {
        Self(response.spend_bundle.aggregated_signature)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WalletSignCoinSpendsApprovalAssetKind {
    Token,
    Nft,
    Did,
    Option,
}

impl From<AssetKind> for WalletSignCoinSpendsApprovalAssetKind {
    fn from(kind: AssetKind) -> Self {
        match kind {
            AssetKind::Token => Self::Token,
            AssetKind::Nft => Self::Nft,
            AssetKind::Did => Self::Did,
            AssetKind::Option => Self::Option,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignCoinSpendsApprovalAsset {
    pub asset_id: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub precision: u8,
    pub kind: WalletSignCoinSpendsApprovalAssetKind,
}

impl From<Asset> for WalletSignCoinSpendsApprovalAsset {
    fn from(asset: Asset) -> Self {
        Self {
            asset_id: asset.asset_id,
            name: asset.name,
            ticker: asset.ticker,
            precision: asset.precision,
            kind: asset.kind.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignCoinSpendsApprovalOutput {
    pub coin_id: String,
    pub amount: WalletSignCoinSpendsAmount,
    pub address: String,
    pub receiving: bool,
    pub burning: bool,
}

impl From<TransactionOutput> for WalletSignCoinSpendsApprovalOutput {
    fn from(output: TransactionOutput) -> Self {
        Self {
            coin_id: output.coin_id,
            amount: output.amount.into(),
            address: output.address,
            receiving: output.receiving,
            burning: output.burning,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignCoinSpendsApprovalInput {
    pub coin_id: String,
    pub amount: WalletSignCoinSpendsAmount,
    pub address: String,
    pub asset: Option<WalletSignCoinSpendsApprovalAsset>,
    pub outputs: Vec<WalletSignCoinSpendsApprovalOutput>,
}

impl From<TransactionInput> for WalletSignCoinSpendsApprovalInput {
    fn from(input: TransactionInput) -> Self {
        Self {
            coin_id: input.coin_id,
            amount: input.amount.into(),
            address: input.address,
            asset: input.asset.map(Into::into),
            outputs: input.outputs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignCoinSpendsApprovalSummary {
    pub fee: WalletSignCoinSpendsAmount,
    pub inputs: Vec<WalletSignCoinSpendsApprovalInput>,
}

impl From<TransactionSummary> for WalletSignCoinSpendsApprovalSummary {
    fn from(summary: TransactionSummary) -> Self {
        Self {
            fee: summary.fee.into(),
            inputs: summary.inputs.into_iter().map(Into::into).collect(),
        }
    }
}

fn validate_params(params: &WalletSignCoinSpendsParams) -> Result<(), BridgeMethodHandleError> {
    if params.coin_spends.is_empty() {
        return Err(BridgeMethodHandleError::invalid_request(
            "coinSpends must contain at least one coin spend",
        ));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for WalletSignCoinSpends {
    fn name(&self) -> &'static str {
        "wallet.signCoinSpends"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletSignCoinSpends)
    }

    fn binds_approval_to_wallet(&self) -> bool {
        true
    }

    async fn prepare_approval(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: WalletSignCoinSpendsParams = parse_required_params(self, request)?;
        validate_params(&params)?;

        let summary = tools
            .app_state
            .lock()
            .await
            .view_coin_spends(params.preview_request())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::invalid_request(format!(
                    "failed to preview coin spends: {err}"
                ))
            })?
            .summary
            .into();

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SignCoinSpends {
                summary,
                partial_sign: params.partial_sign.unwrap_or(false),
            },
        }))
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletSignCoinSpendsParams = parse_required_params(self, request)?;
        validate_params(&params)?;

        let response = tools
            .app_state
            .lock()
            .await
            .sign_coin_spends(params.into_request())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletSignCoinSpendsResult::from(response)))
    }
}
