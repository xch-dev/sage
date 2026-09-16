use async_trait::async_trait;
use sage_api::SendXch;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeApprovalResponse, RustBridgeRequest, UserBridgeCapability,
    parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletSendXch;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSendXchParams {
    pub address: String,
    pub amount: String,
    pub fee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memos: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clawback: Option<u64>,
}

fn approved_send_xch_params(
    approval: &RustBridgeApprovalRequest,
    response: Option<&RustBridgeApprovalResponse>,
) -> Result<WalletSendXchParams, BridgeMethodHandleError> {
    let RustBridgeApprovalBody::SendXch { summary } = &approval.body else {
        return Err(BridgeMethodHandleError::invalid_request(
            "The send XCH approval does not match its bridge request",
        ));
    };

    let response = response.ok_or_else(|| {
        BridgeMethodHandleError::invalid_request(
            "An approved send XCH request requires a selected fee",
        )
    })?;
    let RustBridgeApprovalResponse::SendXch(response) = response;
    let selected_fee = &response.selected_fee;

    if selected_fee.is_empty() || !selected_fee.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(BridgeMethodHandleError::invalid_request(
            "The selected fee must be an integer number of mojos",
        ));
    }

    let selected_fee = selected_fee
        .parse::<u64>()
        .map_err(|_| {
            BridgeMethodHandleError::invalid_request(
                "The selected fee must be an integer number of mojos",
            )
        })?
        .to_string();

    let mut params = summary.clone();
    params.fee = selected_fee;
    Ok(params)
}

fn parse_amount(value: String) -> sage_api::Amount {
    match value.parse::<u64>() {
        Ok(number) => sage_api::Amount::Number(number),
        Err(_) => sage_api::Amount::String(value),
    }
}

impl From<WalletSendXchParams> for SendXch {
    fn from(v: WalletSendXchParams) -> Self {
        Self {
            address: v.address,
            amount: parse_amount(v.amount),
            fee: parse_amount(v.fee),
            memos: v.memos.unwrap_or_default(),
            clawback: v.clawback,
            auto_submit: true,
        }
    }
}

async fn execute_send_xch(
    tools: BridgeTools<'_>,
    params: WalletSendXchParams,
) -> BridgeHandleResult {
    let result = tools
        .app_state
        .lock()
        .await
        .send_xch(params.into())
        .await
        .map_err(|err| {
            BridgeMethodHandleError::internal_error(format!("wallet.sendXch failed: {err}"))
        })?;

    Ok(Box::new(result))
}

#[async_trait]
impl BridgeMethod for WalletSendXch {
    fn name(&self) -> &'static str {
        "wallet.sendXch"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletSendXch)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        if ctx
            .app
            .is_capability_granted(UserBridgeCapability::WalletSendXchAutoSubmit.into())
        {
            return Ok(None);
        }

        let params = parse_required_params::<WalletSendXchParams>(self, request)?;

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SendXch { summary: params },
        }))
    }

    fn binds_approval_to_wallet(&self) -> bool {
        true
    }

    async fn handle_approved(
        &self,
        approval: &RustBridgeApprovalRequest,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
        response: Option<&RustBridgeApprovalResponse>,
    ) -> BridgeHandleResult {
        execute_send_xch(tools, approved_send_xch_params(approval, response)?).await
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        execute_send_xch(tools, parse_required_params(self, request)?).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send_xch_approval() -> RustBridgeApprovalRequest {
        RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SendXch {
                summary: WalletSendXchParams {
                    address: "reviewed-address".to_string(),
                    amount: "123".to_string(),
                    fee: "9000".to_string(),
                    memos: Some(vec!["reviewed-memo".to_string()]),
                    clawback: Some(42),
                },
            },
        }
    }

    fn approval_response(selected_fee: &str) -> RustBridgeApprovalResponse {
        RustBridgeApprovalResponse::SendXch(crate::WalletSendXchApprovalResponse {
            selected_fee: selected_fee.to_string(),
        })
    }

    #[test]
    fn selected_fee_replaces_app_suggestion_and_uses_reviewed_summary() {
        let response = approval_response("0");
        let params = approved_send_xch_params(&send_xch_approval(), Some(&response))
            .expect("selected zero fee should be accepted");

        assert_eq!(params.address, "reviewed-address");
        assert_eq!(params.amount, "123");
        assert_eq!(params.fee, "0");
        assert_eq!(params.memos, Some(vec!["reviewed-memo".to_string()]));
        assert_eq!(params.clawback, Some(42));
    }

    #[test]
    fn selected_fee_is_canonicalized_without_losing_large_values() {
        let response = approval_response("00018446744073709551615");
        let params = approved_send_xch_params(&send_xch_approval(), Some(&response))
            .expect("u64 max fee should be accepted");

        assert_eq!(params.fee, u64::MAX.to_string());
    }

    #[test]
    fn approved_send_xch_requires_a_selected_fee() {
        let error = approved_send_xch_params(&send_xch_approval(), None)
            .expect_err("app suggestion must not be used as a fallback");

        assert!(error.message.contains("requires a selected fee"));
    }

    #[test]
    fn invalid_selected_fees_are_rejected() {
        for fee in ["", "-1", "+1", "1.5", "1e3", " 1", "18446744073709551616"] {
            let response = approval_response(fee);
            assert!(
                approved_send_xch_params(&send_xch_approval(), Some(&response)).is_err(),
                "fee {fee:?} should be rejected"
            );
        }
    }

    #[test]
    fn send_xch_approval_body_must_match_the_method() {
        let approval = RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::OpenExternalUrl {
                url: "https://example.com".to_string(),
            },
        };
        let response = approval_response("0");

        assert!(approved_send_xch_params(&approval, Some(&response)).is_err());
    }

    #[test]
    fn approval_response_rejects_non_fee_fields() {
        let error = serde_json::from_value::<RustBridgeApprovalResponse>(serde_json::json!({
            "kind": "sendXch",
            "response": {
                "selectedFee": "0",
                "address": "attacker-controlled-address"
            }
        }))
        .expect_err("approval responses must reject fields outside their typed variant");

        assert!(error.to_string().contains("unknown field"));
    }
}
