use async_trait::async_trait;
use sage_api::{GetDerivations, GetDerivationsResponse};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    UserBridgeCapability, parse_optional_params,
};

#[derive(Debug, Clone, Copy)]
pub struct WalletGetPublicKeys;

#[derive(Debug, Clone, Copy, Default, Deserialize, Type)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct WalletGetPublicKeysParams {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub hardened: Option<bool>,
}

impl From<WalletGetPublicKeysParams> for GetDerivations {
    fn from(params: WalletGetPublicKeysParams) -> Self {
        Self {
            limit: params.limit.unwrap_or(10),
            offset: params.offset.unwrap_or(0),
            hardened: params.hardened.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(transparent)]
pub struct WalletGetPublicKeysResult(pub Vec<String>);

impl From<GetDerivationsResponse> for WalletGetPublicKeysResult {
    fn from(response: GetDerivationsResponse) -> Self {
        Self(
            response
                .derivations
                .into_iter()
                .map(|derivation| derivation.public_key)
                .collect(),
        )
    }
}

#[async_trait]
impl BridgeMethod for WalletGetPublicKeys {
    fn name(&self) -> &'static str {
        "wallet.getPublicKeys"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::WalletGetPublicKeys)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let _params: WalletGetPublicKeysParams = parse_optional_params(self, request)?;
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: WalletGetPublicKeysParams = parse_optional_params(self, request)?;
        let response = tools
            .app_state
            .lock()
            .await
            .get_derivations(params.into())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(WalletGetPublicKeysResult::from(response)))
    }
}
