use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::event_emit::emit_bridge_event_to_app_id;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{parse_required_params, BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::methods::user::app::resolve_app_base_path;
use crate::bridge::types::RustBridgeApprovalBody;
use crate::lifecycle::update::permissions::grant_requested_capability_internal;
use crate::lifecycle::update::types::GrantCapabilityOutcome;
use crate::permissions::capabilities::{get_user_capability_definition, user_capability_definition_view};

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
        let params: RequestCapabilityGrantParams =
            parse_required_params(self, request)?;

        if ctx
            .app
            .granted_permissions()
            .capabilities
            .contains(&params.capability)
        {
            return Ok(None);
        }

        let definition = get_user_capability_definition(params.capability);

        Ok(Some(RustBridgeApprovalRequest {
            app: ctx.app.clone(),
            source_label: ctx.source_label.to_string(),
            request_id: request.id.clone(),
            body: RustBridgeApprovalBody::CapabilityGrant {
                capability: params.capability,
                definition: user_capability_definition_view(definition),
            },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: RequestCapabilityGrantParams =
            parse_required_params(self, request)?;

        let base_path = resolve_app_base_path(&tools)?;

        let result = match grant_requested_capability_internal(
            &base_path,
            &ctx.app.id(),
            params.capability,
        ) {
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
                let full_granted_capabilities = change.full.clone();

                let _ = emit_bridge_event_to_app_id(
                    tools.app_handle,
                    ctx.app.id(),
                    EventForApp::from_capabilities_change(&request.channel, change),
                )
                    .await;

                RequestCapabilityGrantResult {
                    granted: true,
                    already_granted: None,
                    capability,
                    full_granted_capabilities,
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
