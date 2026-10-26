use async_trait::async_trait;
use sage_api::{Amount, GetNft, GetOption, GetToken, MakeOffer, MakeOfferResponse, OfferAmount};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, parse_required_params,
    unix_timestamp_ms,
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
    /// Accepted only so it can be refused with a clear error: Sage looks up a
    /// requested CAT's hidden puzzle hash itself, and an app must not be able to
    /// swap in a revocable variant of the CAT the user thinks they are receiving.
    #[serde(default, skip_serializing)]
    #[specta(skip)]
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
    /// Unix time in seconds. It must be in the future; omit it for an offer that
    /// never expires.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WalletCreateOfferApprovalAssetKind {
    Xch,
    Cat,
    Nft,
    Option,
}

/// The CAT standard, used to show a CAT amount when the wallet has no record of
/// the CAT and so does not know its precision.
const DEFAULT_CAT_PRECISION: u8 = 3;
const XCH_PRECISION: u8 = 12;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletCreateOfferApprovalAsset {
    pub kind: WalletCreateOfferApprovalAssetKind,
    pub asset_id: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    /// Decimal places used to show `amount`.
    pub precision: u8,
    /// False when the wallet has no record of the CAT and `precision` is the
    /// 3-decimal CAT default.
    pub precision_known: bool,
    /// A non-XCH asset whose name or ticker is "XCH", which the approval must not
    /// let pass for real XCH.
    pub impersonates_xch: bool,
    /// The amount in mojos.
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
    kind: WalletCreateOfferApprovalAssetKind,
    asset_id: Option<String>,
    amount: u64,
}

struct ValidatedOffer {
    offered: Vec<ValidatedLeg>,
    requested: Vec<ValidatedLeg>,
    fee: u64,
    expires_at_second: Option<u64>,
}

/// Classifies an asset id by its form. `make_offer` does the full parsing; this
/// only decides how the approval labels and looks up the asset, and refuses ids
/// that `make_offer` could not accept or that would read as XCH.
fn asset_kind(
    asset_id: Option<&str>,
    field: &str,
) -> Result<WalletCreateOfferApprovalAssetKind, BridgeMethodHandleError> {
    let Some(asset_id) = asset_id else {
        return Ok(WalletCreateOfferApprovalAssetKind::Xch);
    };

    if asset_id.len() == 64 && asset_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        // The all-zero asset id is how the wallet stores XCH itself, so the
        // approval would label it as XCH while the offer treats it as a CAT.
        if asset_id.bytes().all(|byte| byte == b'0') {
            return Err(BridgeMethodHandleError::invalid_request(format!(
                "{field} assetId must not be the zero asset id; omit assetId for XCH"
            )));
        }
        return Ok(WalletCreateOfferApprovalAssetKind::Cat);
    }

    if asset_id.starts_with("nft1") {
        return Ok(WalletCreateOfferApprovalAssetKind::Nft);
    }

    if asset_id.starts_with("option1") {
        return Ok(WalletCreateOfferApprovalAssetKind::Option);
    }

    Err(BridgeMethodHandleError::invalid_request(format!(
        "{field} assetId must be empty for XCH, a CAT asset id, an NFT id or an option id: {asset_id:?}"
    )))
}

fn validate_legs(
    legs: &[WalletCreateOfferAsset],
    field: &str,
) -> Result<Vec<ValidatedLeg>, BridgeMethodHandleError> {
    legs.iter()
        .map(|leg| {
            if leg.hidden_puzzle_hash.is_some() {
                return Err(BridgeMethodHandleError::invalid_request(format!(
                    "{field} hiddenPuzzleHash is not supported: Sage uses the hidden puzzle \
                     hash it already knows for a CAT, so remove hiddenPuzzleHash"
                )));
            }

            let asset_id = leg.asset_id();
            let kind = asset_kind(asset_id.as_deref(), field)?;
            let amount = leg.amount.mojos()?;

            if amount == 0 {
                return Err(BridgeMethodHandleError::invalid_request(format!(
                    "{field} amounts must be greater than zero"
                )));
            }

            if matches!(
                kind,
                WalletCreateOfferApprovalAssetKind::Nft
                    | WalletCreateOfferApprovalAssetKind::Option
            ) && amount != 1
            {
                return Err(BridgeMethodHandleError::invalid_request(format!(
                    "{field} amounts for NFTs and options must be 1"
                )));
            }

            Ok(ValidatedLeg {
                kind,
                asset_id,
                amount,
            })
        })
        .collect()
}

fn validate_params(
    params: &WalletCreateOfferParams,
    now_seconds: u64,
) -> Result<ValidatedOffer, BridgeMethodHandleError> {
    if params.offer_assets.is_empty() && params.request_assets.is_empty() {
        return Err(BridgeMethodHandleError::invalid_request(
            "an offer must offer or request at least one asset",
        ));
    }

    if let Some(expires_at_second) = params.expires_at_second
        && expires_at_second <= now_seconds
    {
        return Err(BridgeMethodHandleError::invalid_request(format!(
            "expiresAtSecond is in the past: {expires_at_second}"
        )));
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

fn now_seconds() -> u64 {
    u64::try_from(unix_timestamp_ms() / 1000).unwrap_or(0)
}

fn offer_amounts(legs: Vec<ValidatedLeg>) -> Vec<OfferAmount> {
    legs.into_iter()
        .map(|leg| OfferAmount {
            asset_id: leg.asset_id,
            // Never taken from the app: make_offer looks up a requested CAT's
            // hidden puzzle hash itself when this is None.
            hidden_puzzle_hash: None,
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

fn is_xch_label(label: Option<&str>) -> bool {
    label.is_some_and(|label| label.trim().eq_ignore_ascii_case("xch"))
}

fn approval_asset_from_labels(
    leg: ValidatedLeg,
    name: Option<String>,
    ticker: Option<String>,
    precision: Option<u8>,
) -> WalletCreateOfferApprovalAsset {
    let (precision, precision_known) = match (leg.kind, precision) {
        (_, Some(precision)) => (precision, true),
        (WalletCreateOfferApprovalAssetKind::Xch, None) => (XCH_PRECISION, true),
        (WalletCreateOfferApprovalAssetKind::Cat, None) => (DEFAULT_CAT_PRECISION, false),
        // NFTs and options are single, indivisible coins.
        (
            WalletCreateOfferApprovalAssetKind::Nft | WalletCreateOfferApprovalAssetKind::Option,
            None,
        ) => (0, true),
    };

    let impersonates_xch = leg.kind != WalletCreateOfferApprovalAssetKind::Xch
        && (is_xch_label(name.as_deref()) || is_xch_label(ticker.as_deref()));

    WalletCreateOfferApprovalAsset {
        kind: leg.kind,
        asset_id: leg.asset_id,
        name,
        ticker,
        precision,
        precision_known,
        impersonates_xch,
        amount: leg.amount.to_string(),
    }
}

async fn approval_asset(
    tools: &BridgeTools<'_>,
    leg: ValidatedLeg,
) -> WalletCreateOfferApprovalAsset {
    // Labels are best effort: a CAT, NFT or option the wallet has never seen has
    // no record, and the approval then shows its kind and id.
    let app_state = tools.app_state.lock().await;

    let (name, ticker, precision) = match leg.kind {
        WalletCreateOfferApprovalAssetKind::Xch | WalletCreateOfferApprovalAssetKind::Cat => {
            let token = app_state
                .get_token(GetToken {
                    asset_id: leg.asset_id.clone(),
                })
                .await
                .ok()
                .and_then(|response| response.token);

            match token {
                Some(token) => (token.name, token.ticker, Some(token.precision)),
                None => (None, None, None),
            }
        }
        WalletCreateOfferApprovalAssetKind::Nft => {
            let name = match leg.asset_id.clone() {
                Some(nft_id) => app_state
                    .get_nft(GetNft { nft_id })
                    .await
                    .ok()
                    .and_then(|response| response.nft)
                    .and_then(|nft| nft.name),
                None => None,
            };
            (name, None, None)
        }
        WalletCreateOfferApprovalAssetKind::Option => {
            let name = match leg.asset_id.clone() {
                Some(option_id) => app_state
                    .get_option(GetOption { option_id })
                    .await
                    .ok()
                    .and_then(|response| response.option)
                    .and_then(|option| option.name),
                None => None,
            };
            (name, None, None)
        }
    };

    drop(app_state);

    approval_asset_from_labels(leg, name, ticker, precision)
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
        let offer = validate_params(&params, now_seconds())?;

        let mut offered = Vec::with_capacity(offer.offered.len());
        for leg in offer.offered {
            offered.push(approval_asset(&tools, leg).await);
        }

        let mut requested = Vec::with_capacity(offer.requested.len());
        for leg in offer.requested {
            requested.push(approval_asset(&tools, leg).await);
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
        let offer = validate_params(&params, now_seconds())?;

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

    const NOW: u64 = 1_800_000_000;
    const CAT_ID: &str = "a628c1c2c6fcb74d53746157e438e108eab5c0bb3e5c80ff9b1910b3e4832913";

    fn params(value: serde_json::Value) -> WalletCreateOfferParams {
        serde_json::from_value(value).expect("params should decode")
    }

    fn validate(value: serde_json::Value) -> Result<ValidatedOffer, BridgeMethodHandleError> {
        validate_params(&params(value), NOW)
    }

    fn error_message(result: Result<ValidatedOffer, BridgeMethodHandleError>) -> String {
        match result {
            Ok(_) => panic!("params should be rejected"),
            Err(error) => format!("{error:?}"),
        }
    }

    fn leg(kind: WalletCreateOfferApprovalAssetKind, asset_id: Option<&str>) -> ValidatedLeg {
        ValidatedLeg {
            kind,
            asset_id: asset_id.map(ToString::to_string),
            amount: 1000,
        }
    }

    #[test]
    fn create_offer_maps_to_an_imported_make_offer() {
        let request = validate(json!({
            "offerAssets": [{ "assetId": "", "amount": 1000 }],
            "requestAssets": [{ "assetId": CAT_ID, "amount": "250" }],
            "fee": "5",
            "expiresAtSecond": 1_900_000_000
        }))
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
            Some(CAT_ID)
        );
        assert_eq!(request.requested_assets[0].hidden_puzzle_hash, None);
        assert!(matches!(
            request.requested_assets[0].amount,
            Amount::Number(250)
        ));
    }

    #[test]
    fn missing_fee_is_zero() {
        let offer = validate(json!({
            "offerAssets": [{ "amount": 1 }],
            "requestAssets": []
        }))
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
            let result = validate(json!({
                "offerAssets": [{ "assetId": "", "amount": amount }],
                "requestAssets": []
            }));

            assert!(result.is_err(), "amount {amount} should be rejected");
        }
    }

    #[test]
    fn invalid_fee_is_rejected() {
        let result = validate(json!({
            "offerAssets": [{ "amount": 1 }],
            "requestAssets": [],
            "fee": "1.5"
        }));

        assert!(result.is_err());
    }

    #[test]
    fn empty_offer_is_rejected() {
        let result = validate(json!({
            "offerAssets": [],
            "requestAssets": []
        }));

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

    #[test]
    fn hidden_puzzle_hash_is_rejected() {
        for field in ["offerAssets", "requestAssets"] {
            let other = if field == "offerAssets" {
                "requestAssets"
            } else {
                "offerAssets"
            };
            let message = error_message(validate(json!({
                field: [{
                    "assetId": CAT_ID,
                    "hiddenPuzzleHash": "00".repeat(32),
                    "amount": 1000
                }],
                other: []
            })));

            assert!(message.contains("hiddenPuzzleHash"), "{message}");
        }
    }

    #[test]
    fn past_expiry_is_rejected() {
        for expires_at_second in [0, NOW - 1, NOW] {
            let message = error_message(validate(json!({
                "offerAssets": [{ "amount": 1 }],
                "requestAssets": [],
                "expiresAtSecond": expires_at_second
            })));

            assert!(message.contains("expiresAtSecond"), "{message}");
        }

        let offer = validate(json!({
            "offerAssets": [{ "amount": 1 }],
            "requestAssets": [],
            "expiresAtSecond": NOW + 1
        }))
        .expect("future expiry is valid");
        assert_eq!(offer.expires_at_second, Some(NOW + 1));
    }

    #[test]
    fn asset_kinds_are_classified() {
        let nft_id = "nft1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
        let option_id = "option1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";

        let offer = validate(json!({
            "offerAssets": [{ "amount": 1 }, { "assetId": CAT_ID, "amount": 1 }],
            "requestAssets": [
                { "assetId": nft_id, "amount": 1 },
                { "assetId": option_id, "amount": "1" }
            ]
        }))
        .expect("valid params");

        let kinds = offer
            .offered
            .iter()
            .chain(&offer.requested)
            .map(|leg| leg.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                WalletCreateOfferApprovalAssetKind::Xch,
                WalletCreateOfferApprovalAssetKind::Cat,
                WalletCreateOfferApprovalAssetKind::Nft,
                WalletCreateOfferApprovalAssetKind::Option,
            ]
        );

        for (asset_id, amount) in [
            (nft_id, 2),
            (option_id, 2),
            ("00".repeat(32).as_str(), 1),
            ("xch", 1),
            ("did:chia:1abc", 1),
        ] {
            let result = validate(json!({
                "offerAssets": [{ "assetId": asset_id, "amount": amount }],
                "requestAssets": []
            }));
            assert!(result.is_err(), "{asset_id} x{amount} should be rejected");
        }
    }

    #[test]
    fn non_xch_assets_labelled_xch_are_flagged() {
        for (name, ticker) in [
            (None, Some("XCH")),
            (None, Some("xch")),
            (Some(" Xch "), None),
            (Some("XCH"), Some("FAKE")),
        ] {
            for kind in [
                WalletCreateOfferApprovalAssetKind::Cat,
                WalletCreateOfferApprovalAssetKind::Nft,
                WalletCreateOfferApprovalAssetKind::Option,
            ] {
                let asset = approval_asset_from_labels(
                    leg(kind, Some(CAT_ID)),
                    name.map(ToString::to_string),
                    ticker.map(ToString::to_string),
                    Some(3),
                );
                assert!(asset.impersonates_xch, "{kind:?} {name:?} {ticker:?}");
            }
        }

        let real = approval_asset_from_labels(
            leg(WalletCreateOfferApprovalAssetKind::Xch, None),
            Some("Chia".to_string()),
            Some("XCH".to_string()),
            Some(12),
        );
        assert!(!real.impersonates_xch);

        let other = approval_asset_from_labels(
            leg(WalletCreateOfferApprovalAssetKind::Cat, Some(CAT_ID)),
            Some("Wrapped XCH".to_string()),
            Some("wXCH".to_string()),
            Some(3),
        );
        assert!(!other.impersonates_xch);
    }

    #[test]
    fn unknown_cats_default_to_three_decimals() {
        let unknown = approval_asset_from_labels(
            leg(WalletCreateOfferApprovalAssetKind::Cat, Some(CAT_ID)),
            None,
            None,
            None,
        );
        assert_eq!(unknown.precision, DEFAULT_CAT_PRECISION);
        assert!(!unknown.precision_known);
        assert_eq!(unknown.amount, "1000");

        let known = approval_asset_from_labels(
            leg(WalletCreateOfferApprovalAssetKind::Cat, Some(CAT_ID)),
            None,
            Some("SBX".to_string()),
            Some(4),
        );
        assert_eq!(known.precision, 4);
        assert!(known.precision_known);

        let nft = approval_asset_from_labels(
            leg(WalletCreateOfferApprovalAssetKind::Nft, Some("nft1abc")),
            None,
            None,
            None,
        );
        assert_eq!(nft.precision, 0);
        assert!(nft.precision_known);
    }
}
