use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::types::RustBridgeApprovalBody;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use sage_api::SendXch;

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
        if ctx.app.is_capability_granted(UserBridgeCapability::WalletSendXchAutoSubmit) {
            return Ok(None);
        }

        let params = parse_required_params::<WalletSendXchParams>(self, request)?;

        Ok(Some(RustBridgeApprovalRequest {
            app: ctx.app.clone(),
            source_label: ctx.source_label.to_string(),
            request_id: request.id.clone(),
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

        Ok(Box::new(result))
    }
}
