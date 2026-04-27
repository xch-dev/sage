use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::event_emit::emit_bridge_event_to_app_id;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, parse_required_params,
};
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::methods::user::app::resolve_app_base_path;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::types::RustBridgeApprovalBody;
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::lifecycle::update::permissions::grant_requested_network_whitelist_entry_internal;
use crate::lifecycle::update::types::GrantNetworkWhitelistOutcome;
use crate::types::SageNetworkWhitelistEntry;

#[derive(Debug, Clone, Copy)]
pub struct AppRequestNetworkWhitelistGrant;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestNetworkWhitelistGrantParams {
    pub entry: SageNetworkWhitelistEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestNetworkWhitelistGrantResult {
    pub granted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub already_granted: Option<bool>,
    pub entry: SageNetworkWhitelistEntry,
    pub full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
}

#[async_trait]
impl BridgeMethod for AppRequestNetworkWhitelistGrant {
    fn name(&self) -> &'static str {
        "app.requestNetworkWhitelistGrant"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppRequestNetworkWhitelistGrant)
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: RequestNetworkWhitelistGrantParams = parse_required_params(self, request)?;

        if ctx
            .app
            .granted_permissions()
            .network()
            .whitelist()
            .any(|entry| entry == &params.entry)
        {
            return Ok(None);
        }

        Ok(Some(RustBridgeApprovalRequest {
            app: ctx.app.clone(),
            source_label: ctx.source_label.to_string(),
            request_id: request.id.clone(),
            body: RustBridgeApprovalBody::NetworkWhitelistGrant {
                entry: params.entry,
            },
        }))
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: RequestNetworkWhitelistGrantParams = parse_required_params(self, request)?;

        let base_path = resolve_app_base_path(&tools)?;

        let result = match grant_requested_network_whitelist_entry_internal(
            &base_path,
            ctx.app.id(),
            &params.entry,
        ) {
            Ok(GrantNetworkWhitelistOutcome::AlreadyGranted {
                entry,
                full_granted_network_whitelist,
            }) => RequestNetworkWhitelistGrantResult {
                granted: true,
                already_granted: Some(true),
                entry,
                full_granted_network_whitelist,
            },

            Ok(GrantNetworkWhitelistOutcome::Granted { entry, change }) => {
                let full = change.full.clone();

                let _ = emit_bridge_event_to_app_id(
                    tools.app_handle,
                    ctx.app.id(),
                    EventForApp::from_network_whitelist_change(&request.channel, change),
                )
                .await;

                RequestNetworkWhitelistGrantResult {
                    granted: true,
                    already_granted: None,
                    entry,
                    full_granted_network_whitelist: full,
                }
            }

            Err(err) => {
                return Err(BridgeMethodHandleError::internal_error(format!(
                    "failed to grant requested network whitelist entry: {err}"
                )));
            }
        };

        Ok(Box::new(result))
    }
}
