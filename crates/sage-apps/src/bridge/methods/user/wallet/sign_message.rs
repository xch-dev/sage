use async_trait::async_trait;
use sage_api::wallet_connect::{SignMessageWithPublicKey, SignMessageWithPublicKeyResponse};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, parse_required_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletSignMessage;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletSignMessageParams {
    pub message: String,
    pub public_key: String,
}

impl From<WalletSignMessageParams> for SignMessageWithPublicKey {
    fn from(params: WalletSignMessageParams) -> Self {
        Self {
            message: params.message,
            public_key: params.public_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(transparent)]
pub struct WalletSignMessageResult(pub String);

impl From<SignMessageWithPublicKeyResponse> for WalletSignMessageResult {
    fn from(response: SignMessageWithPublicKeyResponse) -> Self {
        Self(response.signature)
    }
}

#[async_trait]
impl BridgeMethod for WalletSignMessage {
    fn name(&self) -> &'static str {
        "wallet.signMessage"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletSignMessage)
    }

    fn binds_approval_to_wallet(&self) -> bool {
        true
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: WalletSignMessageParams = parse_required_params(self, request)?;

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::SignMessage {
                message: params.message,
                public_key: params.public_key,
            },
        }))
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletSignMessageParams = parse_required_params(self, request)?;
        let response = tools
            .app_state
            .lock()
            .await
            .sign_message_with_public_key(params.into())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletSignMessageResult::from(response)))
    }
}
