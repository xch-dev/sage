use async_trait::async_trait;
use sage_api::{HandleConfirmation, RecipientErrorKind, ResolveHandle, SendXch};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeApprovalResponse, RustBridgeRequest, UserBridgeCapability,
    WalletSendXchHandle, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletSendXch;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WalletSendXchParams {
    /// An address, or an XCHandles handle written as `@handle`. Sending to a handle always
    /// asks the user for approval, so that they can review the verified address it resolves
    /// to. It fails with `recipient_changed` if the handle changed since it was last sent to
    /// and the user didn't confirm the new address, or `recipient_unresolved` if it can't be
    /// verified.
    pub address: String,
    pub amount: String,
    pub fee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memos: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clawback: Option<u64>,
}

/// Whether the recipient is an XCHandles handle, which is resolved by Sage.
fn is_handle_recipient(params: &WalletSendXchParams) -> bool {
    params.address.trim().starts_with('@')
}

/// The send XCH parameters the user approved, and the address they reviewed a handle
/// recipient as resolving to.
fn approved_send_xch_params(
    approval: &RustBridgeApprovalRequest,
    response: Option<&RustBridgeApprovalResponse>,
) -> Result<(WalletSendXchParams, Option<String>), BridgeMethodHandleError> {
    let RustBridgeApprovalBody::SendXch { summary, .. } = &approval.body else {
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

    let confirmed_address = if is_handle_recipient(&params) {
        response.confirmed_address.clone()
    } else {
        None
    };

    Ok((params, confirmed_address))
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
            coin_ids: None,
            confirmed_handles: Vec::new(),
        }
    }
}

async fn execute_send_xch(
    tools: BridgeTools<'_>,
    params: WalletSendXchParams,
    confirmed_address: Option<String>,
) -> BridgeHandleResult {
    let mut request = SendXch::from(params);

    if let Some(address) = confirmed_address {
        request.confirmed_handles.push(HandleConfirmation {
            handle: request.address.trim().to_string(),
            address,
        });
    }

    let result = tools
        .app_state
        .lock()
        .await
        .send_xch(request)
        .await
        .map_err(|err| {
            let message = format!("wallet.sendXch failed: {err}");

            // Apps can tell a handle which changed hands, which the user has to confirm in
            // Sage, from one which couldn't be resolved.
            match err.recipient_errors() {
                Some(errors)
                    if errors
                        .iter()
                        .any(|error| error.kind == RecipientErrorKind::Changed) =>
                {
                    BridgeMethodHandleError::new("recipient_changed", message)
                }
                Some(_) => BridgeMethodHandleError::new("recipient_unresolved", message),
                None => BridgeMethodHandleError::internal_error(message),
            }
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
        let params = parse_required_params::<WalletSendXchParams>(self, request)?;

        // A handle is always reviewed by the user, even with auto submit, since it can
        // change hands and only Sage can show what it resolves to now.
        if !is_handle_recipient(&params)
            && ctx
                .app
                .is_capability_granted(UserBridgeCapability::WalletSendXchAutoSubmit.into())
        {
            return Ok(None);
        }

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SendXch {
                summary: params,
                handle: None,
            },
        }))
    }

    async fn prepare_approval(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let Some(mut approval) = self.approval_request(ctx, request)? else {
            return Ok(None);
        };

        if let RustBridgeApprovalBody::SendXch { summary, handle } = &mut approval.body
            && is_handle_recipient(summary)
        {
            let result = tools
                .app_state
                .lock()
                .await
                .resolve_handle(ResolveHandle {
                    handle: summary.address.trim().to_string(),
                })
                .await;

            *handle = Some(Box::new(match result {
                Ok(response) => WalletSendXchHandle {
                    record: Some(response.handle),
                    error: None,
                },
                Err(error) => WalletSendXchHandle {
                    record: None,
                    error: Some(error.to_string()),
                },
            }));
        }

        Ok(Some(approval))
    }

    fn binds_approval_to_wallet(&self) -> bool {
        true
    }

    fn requires_signing(&self) -> bool {
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
        let (params, confirmed_address) = approved_send_xch_params(approval, response)?;
        execute_send_xch(tools, params, confirmed_address).await
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletSendXchParams = parse_required_params(self, request)?;

        // Handles always require approval, so this can't be reached with one, but the
        // approval is what confirms the handle's address.
        if is_handle_recipient(&params) {
            return Err(BridgeMethodHandleError::invalid_request(
                "Sending to a handle requires approval",
            ));
        }

        execute_send_xch(tools, params, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send_xch_approval() -> RustBridgeApprovalRequest {
        RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SendXch {
                handle: None,
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
            confirmed_address: None,
        })
    }

    #[test]
    fn selected_fee_replaces_app_suggestion_and_uses_reviewed_summary() {
        let response = approval_response("0");
        let (params, _) = approved_send_xch_params(&send_xch_approval(), Some(&response))
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
        let (params, _) = approved_send_xch_params(&send_xch_approval(), Some(&response))
            .expect("u64 max fee should be accepted");

        assert_eq!(params.fee, u64::MAX.to_string());
    }

    #[test]
    fn confirmed_address_only_applies_to_handle_recipients() {
        let response = RustBridgeApprovalResponse::SendXch(crate::WalletSendXchApprovalResponse {
            selected_fee: "0".to_string(),
            confirmed_address: Some("xch1reviewed".to_string()),
        });

        let (_, confirmed) = approved_send_xch_params(&send_xch_approval(), Some(&response))
            .expect("approval should be accepted");
        assert_eq!(confirmed, None);

        let mut approval = send_xch_approval();
        let RustBridgeApprovalBody::SendXch { summary, .. } = &mut approval.body else {
            unreachable!();
        };
        summary.address = "@alice".to_string();

        let (params, confirmed) = approved_send_xch_params(&approval, Some(&response))
            .expect("approval should be accepted");
        assert_eq!(params.address, "@alice");
        assert_eq!(confirmed.as_deref(), Some("xch1reviewed"));
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
