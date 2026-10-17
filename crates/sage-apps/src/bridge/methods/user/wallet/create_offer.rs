use async_trait::async_trait;
use sage_api::{Amount, GetToken, MakeOffer, MakeOfferResponse, OfferAmount};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletCreateOffer;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(untagged)]
pub enum WalletCreateOfferAmount {
    String(String),
    Number(u64),
}

impl WalletCreateOfferAmount {
    /// The amount as an integer number of mojos, so the approval shows exactly
    /// what the wallet will be asked to offer or request.
    fn mojos(&self) -> Result<u64, BridgeMethodHandleError> {
        match self {
            Self::Number(value) => Ok(*value),
            Self::String(value) => {
                if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(BridgeMethodHandleError::invalid_request(format!(
                        "amount must be an integer number of mojos: {value:?}"
                    )));
                }

                value.parse::<u64>().map_err(|_| {
                    BridgeMethodHandleError::invalid_request(format!(
                        "amount is out of range: {value}"
                    ))
                })
            }
        }
    }
}

/// One asset leg of an offer. An empty or missing `assetId` is XCH; otherwise it
/// is a CAT asset id, an NFT id or an option id, as `chia_createOffer` accepts.
#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WalletCreateOfferAsset {
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden_puzzle_hash: Option<String>,
    pub amount: WalletCreateOfferAmount,
}

impl WalletCreateOfferAsset {
    fn asset_id(&self) -> Option<String> {
        self.asset_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(ToString::to_string)
    }
}

/// Mirrors the `WalletConnect` command `chia_createOffer`, plus an optional expiration.
#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WalletCreateOfferParams {
    pub offer_assets: Vec<WalletCreateOfferAsset>,
    pub request_assets: Vec<WalletCreateOfferAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<WalletCreateOfferAmount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_second: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletCreateOfferResult {
    pub offer: String,
    pub id: String,
}

impl From<MakeOfferResponse> for WalletCreateOfferResult {
    fn from(response: MakeOfferResponse) -> Self {
        Self {
            offer: response.offer,
            id: response.offer_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletCreateOfferApprovalAsset {
    pub asset_id: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub precision: Option<u8>,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletCreateOfferApprovalSummary {
    pub offered: Vec<WalletCreateOfferApprovalAsset>,
    pub requested: Vec<WalletCreateOfferApprovalAsset>,
    pub fee: String,
    pub expires_at_second: Option<u64>,
}

struct ValidatedLeg {
    asset_id: Option<String>,
    hidden_puzzle_hash: Option<String>,
    amount: u64,
}

struct ValidatedOffer {
    offered: Vec<ValidatedLeg>,
    requested: Vec<ValidatedLeg>,
    fee: u64,
    expires_at_second: Option<u64>,
}

fn validate_legs(
    legs: &[WalletCreateOfferAsset],
    field: &str,
) -> Result<Vec<ValidatedLeg>, BridgeMethodHandleError> {
    legs.iter()
        .map(|leg| {
            let amount = leg.amount.mojos()?;

            if amount == 0 {
                return Err(BridgeMethodHandleError::invalid_request(format!(
                    "{field} amounts must be greater than zero"
                )));
            }

            Ok(ValidatedLeg {
                asset_id: leg.asset_id(),
                hidden_puzzle_hash: leg.hidden_puzzle_hash.clone(),
                amount,
            })
        })
        .collect()
}

fn validate_params(
    params: &WalletCreateOfferParams,
) -> Result<ValidatedOffer, BridgeMethodHandleError> {
    if params.offer_assets.is_empty() && params.request_assets.is_empty() {
        return Err(BridgeMethodHandleError::invalid_request(
            "an offer must offer or request at least one asset",
        ));
    }

    Ok(ValidatedOffer {
        offered: validate_legs(&params.offer_assets, "offerAssets")?,
        requested: validate_legs(&params.request_assets, "requestAssets")?,
        fee: params
            .fee
            .as_ref()
            .map(WalletCreateOfferAmount::mojos)
            .transpose()?
            .unwrap_or(0),
        expires_at_second: params.expires_at_second,
    })
}

fn offer_amounts(legs: Vec<ValidatedLeg>) -> Vec<OfferAmount> {
    legs.into_iter()
        .map(|leg| OfferAmount {
            asset_id: leg.asset_id,
            hidden_puzzle_hash: leg.hidden_puzzle_hash,
            amount: Amount::Number(leg.amount),
        })
        .collect()
}

impl ValidatedOffer {
    fn into_request(self) -> MakeOffer {
        MakeOffer {
            requested_assets: offer_amounts(self.requested),
            offered_assets: offer_amounts(self.offered),
            fee: Amount::Number(self.fee),
            receive_address: None,
            expires_at_second: self.expires_at_second,
            // Import it, so the offer is listed on the Offers page, can be cancelled
            // there, and its coins are reserved like any offer made in Sage.
            auto_import: true,
            coin_ids: None,
        }
    }
}

async fn approval_asset(
    tools: &BridgeTools<'_>,
    asset_id: Option<String>,
    amount: u64,
) -> WalletCreateOfferApprovalAsset {
    // Labels are best effort: a CAT the wallet has never seen, an NFT or an option
    // has no token record, and the approval then shows the id alone.
    let token = tools
        .app_state
        .lock()
        .await
        .get_token(GetToken {
            asset_id: asset_id.clone(),
        })
        .await
        .ok()
        .and_then(|response| response.token);

    WalletCreateOfferApprovalAsset {
        asset_id,
        name: token.as_ref().and_then(|token| token.name.clone()),
        ticker: token.as_ref().and_then(|token| token.ticker.clone()),
        precision: token.as_ref().map(|token| token.precision),
        amount: amount.to_string(),
    }
}

#[async_trait]
impl BridgeMethod for WalletCreateOffer {
    fn name(&self) -> &'static str {
        "wallet.createOffer"
    }

    // Making an offer is Sage signing spends of its own coins, so it sits under
    // the same capability as signing an app's coin spends, with the same
    // per-request approval. It can never be auto-approved.
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
        let params: WalletCreateOfferParams = parse_required_params(self, request)?;
        let offer = validate_params(&params)?;

        let mut offered = Vec::with_capacity(offer.offered.len());
        for leg in offer.offered {
            offered.push(approval_asset(&tools, leg.asset_id, leg.amount).await);
        }

        let mut requested = Vec::with_capacity(offer.requested.len());
        for leg in offer.requested {
            requested.push(approval_asset(&tools, leg.asset_id, leg.amount).await);
        }

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::CreateOffer {
                summary: WalletCreateOfferApprovalSummary {
                    offered,
                    requested,
                    fee: offer.fee.to_string(),
                    expires_at_second: offer.expires_at_second,
                },
            },
        }))
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletCreateOfferParams = parse_required_params(self, request)?;
        let offer = validate_params(&params)?;

        let response = tools
            .app_state
            .lock()
            .await
            .make_offer(offer.into_request())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletCreateOfferResult::from(response)))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn params(value: serde_json::Value) -> WalletCreateOfferParams {
        serde_json::from_value(value).expect("params should decode")
    }

    #[test]
    fn create_offer_maps_to_an_imported_make_offer() {
        let request = validate_params(&params(json!({
            "offerAssets": [{ "assetId": "", "amount": 1000 }],
            "requestAssets": [{
                "assetId": "a628c1c2c6fcb74d53746157e438e108eab5c0bb3e5c80ff9b1910b3e4832913",
                "hiddenPuzzleHash": "00".repeat(32),
                "amount": "250"
            }],
            "fee": "5",
            "expiresAtSecond": 1_900_000_000
        })))
        .expect("valid params")
        .into_request();

        assert!(request.auto_import);
        assert!(request.coin_ids.is_none());
        assert!(request.receive_address.is_none());
        assert_eq!(request.expires_at_second, Some(1_900_000_000));
        assert!(matches!(request.fee, Amount::Number(5)));

        assert_eq!(request.offered_assets.len(), 1);
        assert_eq!(request.offered_assets[0].asset_id, None);
        assert!(matches!(
            request.offered_assets[0].amount,
            Amount::Number(1000)
        ));

        assert_eq!(request.requested_assets.len(), 1);
        assert_eq!(
            request.requested_assets[0].asset_id.as_deref(),
            Some("a628c1c2c6fcb74d53746157e438e108eab5c0bb3e5c80ff9b1910b3e4832913")
        );
        assert_eq!(
            request.requested_assets[0].hidden_puzzle_hash.as_deref(),
            Some("00".repeat(32).as_str())
        );
        assert!(matches!(
            request.requested_assets[0].amount,
            Amount::Number(250)
        ));
    }

    #[test]
    fn missing_fee_is_zero() {
        let offer = validate_params(&params(json!({
            "offerAssets": [{ "amount": 1 }],
            "requestAssets": []
        })))
        .expect("valid params");

        assert_eq!(offer.fee, 0);
        assert_eq!(offer.offered[0].asset_id, None);
    }

    #[test]
    fn invalid_amounts_are_rejected() {
        for amount in [
            json!(0),
            json!("0"),
            json!(""),
            json!("-1"),
            json!("1.5"),
            json!("1e3"),
            json!(" 1"),
            json!("18446744073709551616"),
        ] {
            let result = validate_params(&params(json!({
                "offerAssets": [{ "assetId": "", "amount": amount }],
                "requestAssets": []
            })));

            assert!(result.is_err(), "amount {amount} should be rejected");
        }
    }

    #[test]
    fn invalid_fee_is_rejected() {
        let result = validate_params(&params(json!({
            "offerAssets": [{ "amount": 1 }],
            "requestAssets": [],
            "fee": "1.5"
        })));

        assert!(result.is_err());
    }

    #[test]
    fn empty_offer_is_rejected() {
        let result = validate_params(&params(json!({
            "offerAssets": [],
            "requestAssets": []
        })));

        assert!(result.is_err());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let error = serde_json::from_value::<WalletCreateOfferParams>(json!({
            "offerAssets": [],
            "requestAssets": [],
            "coinIds": ["coin-id"]
        }))
        .expect_err("unknown top-level fields must be rejected");
        assert!(error.to_string().contains("unknown field"));

        let error = serde_json::from_value::<WalletCreateOfferParams>(json!({
            "offerAssets": [{ "amount": 1, "puzzleHash": "00" }],
            "requestAssets": []
        }))
        .expect_err("unknown leg fields must be rejected");
        assert!(error.to_string().contains("unknown field"));
    }
}
