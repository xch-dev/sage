use sage_api::SendXch;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, bridge_result,
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

impl BridgeMethodHandler for WalletSendXch {
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

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletSendXchParams = parse_required_params(self, request)?;
        let req: SendXch = params.into();

        let result = tools
            .app_state
            .lock()
            .await
            .send_xch(req)
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        bridge_result(result)
    }
}
