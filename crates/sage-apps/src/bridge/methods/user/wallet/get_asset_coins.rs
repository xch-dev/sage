use async_trait::async_trait;
use sage_api::Amount;
use sage_api::wallet_connect::{
    AssetCoinType, GetAssetCoins, GetAssetCoinsResponse, LineageProof, SpendableCoin,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetAssetCoins;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WalletAssetCoinType {
    Cat,
    Did,
    Nft,
}

impl From<WalletAssetCoinType> for AssetCoinType {
    fn from(kind: WalletAssetCoinType) -> Self {
        match kind {
            WalletAssetCoinType::Cat => Self::Cat,
            WalletAssetCoinType::Did => Self::Did,
            WalletAssetCoinType::Nft => Self::Nft,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletGetAssetCoinsParams {
    #[serde(default, rename = "type")]
    pub kind: Option<WalletAssetCoinType>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub included_locked: Option<bool>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub limit: Option<u32>,
}

impl From<WalletGetAssetCoinsParams> for GetAssetCoins {
    fn from(params: WalletGetAssetCoinsParams) -> Self {
        Self {
            kind: params.kind.map(Into::into),
            asset_id: params.asset_id,
            included_locked: params.included_locked,
            offset: params.offset,
            limit: params.limit,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(untagged)]
pub enum WalletAssetAmount {
    String(String),
    Number(u64),
}

impl From<Amount> for WalletAssetAmount {
    fn from(amount: Amount) -> Self {
        match amount {
            Amount::String(value) => Self::String(value),
            Amount::Number(value) => Self::Number(value),
        }
    }
}

impl From<WalletAssetAmount> for Amount {
    fn from(amount: WalletAssetAmount) -> Self {
        match amount {
            WalletAssetAmount::String(value) => Self::String(value),
            WalletAssetAmount::Number(value) => Self::Number(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(deny_unknown_fields)]
pub struct WalletAssetCoin {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: WalletAssetAmount,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletAssetLineageProof {
    pub parent_name: Option<String>,
    pub inner_puzzle_hash: Option<String>,
    pub amount: Option<WalletAssetAmount>,
}

impl From<LineageProof> for WalletAssetLineageProof {
    fn from(proof: LineageProof) -> Self {
        Self {
            parent_name: proof.parent_name,
            inner_puzzle_hash: proof.inner_puzzle_hash,
            amount: proof.amount.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletSpendableAssetCoin {
    pub coin: WalletAssetCoin,
    pub coin_name: String,
    pub puzzle: String,
    pub confirmed_block_index: u32,
    pub locked: bool,
    pub lineage_proof: Option<WalletAssetLineageProof>,
}

impl From<SpendableCoin> for WalletSpendableAssetCoin {
    fn from(coin: SpendableCoin) -> Self {
        Self {
            coin: WalletAssetCoin {
                parent_coin_info: coin.coin.parent_coin_info,
                puzzle_hash: coin.coin.puzzle_hash,
                amount: coin.coin.amount.into(),
            },
            coin_name: coin.coin_name,
            puzzle: coin.puzzle,
            confirmed_block_index: coin.confirmed_block_index,
            locked: coin.locked,
            lineage_proof: coin.lineage_proof.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(transparent)]
pub struct WalletGetAssetCoinsResult(pub Vec<WalletSpendableAssetCoin>);

impl From<GetAssetCoinsResponse> for WalletGetAssetCoinsResult {
    fn from(coins: GetAssetCoinsResponse) -> Self {
        Self(coins.into_iter().map(Into::into).collect())
    }
}

fn validate_params(params: &WalletGetAssetCoinsParams) -> Result<(), BridgeMethodHandleError> {
    if params.kind == Some(WalletAssetCoinType::Cat) && params.asset_id.is_none() {
        return Err(BridgeMethodHandleError::invalid_request(
            "assetId is required when type is cat",
        ));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for WalletGetAssetCoins {
    fn name(&self) -> &'static str {
        "wallet.getAssetCoins"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetAssetCoins)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: WalletGetAssetCoinsParams = parse_required_params(self, request)?;
        validate_params(&params)?;
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletGetAssetCoinsParams = parse_required_params(self, request)?;
        validate_params(&params)?;

        let response = tools
            .app_state
            .lock()
            .await
            .get_asset_coins(params.into())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletGetAssetCoinsResult::from(response)))
    }
}
