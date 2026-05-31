use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeApprovalBody;
use crate::bridge::resolve_app_base_path;
use crate::bridge::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::capabilities::UserBridgeCapability;
use crate::capabilities::get_user_capability_definition;
use crate::lifecycle::GrantCapabilityOutcome;
use crate::lifecycle::grant_capability;

#[derive(Debug, Clone, Copy)]
pub struct AppRequestCapabilityGrant;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestCapabilityGrantParams {
    pub capability: UserBridgeCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestCapabilityGrantResult {
    pub granted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub already_granted: Option<bool>,
    pub capability: UserBridgeCapability,
    pub full_granted_capabilities: Vec<UserBridgeCapability>,
}

fn ensure_capability_requestable_by_app(
    capability: UserBridgeCapability,
) -> Result<(), BridgeMethodHandleError> {
    let definition = get_user_capability_definition(capability);

    if !definition.flags().requestable_by_app() {
        return Err(BridgeMethodHandleError::invalid_request(format!(
            "capability cannot be requested by app: {}",
            capability.key()
        )));
    }

    Ok(())
}

#[async_trait]
impl BridgeMethod for AppRequestCapabilityGrant {
    fn name(&self) -> &'static str {
        "app.requestCapabilityGrant"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppRequestCapabilityGrant)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: RequestCapabilityGrantParams = parse_required_params(self, request)?;

        ensure_capability_requestable_by_app(params.capability)?;

        if ctx.app.is_capability_granted(params.capability.into()) {
            return Ok(None);
        }

        let definition = get_user_capability_definition(params.capability);

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::CapabilityGrant {
                capability: params.capability,
                definition: definition.into(),
            },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: RequestCapabilityGrantParams = parse_required_params(self, request)?;

        ensure_capability_requestable_by_app(params.capability)?;

        let base_path = resolve_app_base_path(&tools)?;

        let result = match grant_capability(
            tools.app_handle,
            tools.host_state,
            &base_path,
            &ctx.app.id(),
            params.capability,
        )
        .await
        {
            Ok(GrantCapabilityOutcome::AlreadyGranted {
                capability,
                full_granted_capabilities,
            }) => RequestCapabilityGrantResult {
                granted: true,
                already_granted: Some(true),
                capability,
                full_granted_capabilities,
            },

            Ok(GrantCapabilityOutcome::Granted { capability, change }) => {
                RequestCapabilityGrantResult {
                    granted: true,
                    already_granted: None,
                    capability,
                    full_granted_capabilities: change.full,
                }
            }

            Err(err) => {
                return Err(BridgeMethodHandleError::internal_error(format!(
                    "failed to grant requested capability: {err}"
                )));
            }
        };

        Ok(Box::new(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_requestable_capability() {
        let err =
            ensure_capability_requestable_by_app(UserBridgeCapability::WalletSendXchAutoSubmit)
                .expect_err("auto-submit send must not be requestable by running apps");

        let message = format!("{err:?}");

        assert!(
            message.contains("wallet.send_xch_auto_submit"),
            "error should mention rejected capability, got: {message}"
        );
    }

    #[test]
    fn allows_requestable_capability() {
        ensure_capability_requestable_by_app(UserBridgeCapability::WalletSendXch)
            .expect("regular send capability should be requestable by running apps");
    }
}
