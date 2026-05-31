use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, GrantNetworkWhitelistOutcome,
    RustBridgeApprovalBody, RustBridgeApprovalRequest, RustBridgeRequest,
    SageNetworkWhitelistEntry, UserBridgeCapability, bridge_result, grant_network_whitelist_entry,
    parse_required_params, resolve_app_base_path,
};

#[derive(Debug, Clone, Copy)]
pub struct AppRequestNetworkWhitelistGrant;

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestNetworkWhitelistGrantParams {
    pub entry: SageNetworkWhitelistEntry,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RequestNetworkWhitelistGrantResult {
    pub granted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub already_granted: Option<bool>,

    pub entry: SageNetworkWhitelistEntry,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    pub full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
}

impl BridgeMethodHandler for AppRequestNetworkWhitelistGrant {
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

        let already_granted = ctx.app.with(|app| {
            let network = app.granted_permissions().network();

            match params.network_id.as_deref() {
                Some(network_id) => network
                    .whitelist_by_network()
                    .get(network_id)
                    .is_some_and(|entries| entries.contains(&params.entry)),
                None => network.whitelist_iter().any(|entry| entry == &params.entry),
            }
        });

        if already_granted {
            return Ok(None);
        }

        Ok(Some(RustBridgeApprovalRequest {
            body: RustBridgeApprovalBody::NetworkWhitelistGrant {
                entry: params.entry,
                network_id: params.network_id,
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

        let grant_result = grant_network_whitelist_entry(
            tools.app_handle,
            tools.host_state,
            &base_path,
            &ctx.app.id(),
            params.network_id.as_deref(),
            &params.entry,
        )
        .await;

        let result = match grant_result {
            Ok(GrantNetworkWhitelistOutcome::AlreadyGranted {
                entry,
                full_granted_network_whitelist,
            }) => RequestNetworkWhitelistGrantResult {
                granted: true,
                already_granted: Some(true),
                entry,
                network_id: params.network_id,
                full_granted_network_whitelist,
            },

            Ok(GrantNetworkWhitelistOutcome::Granted { entry, change }) => {
                RequestNetworkWhitelistGrantResult {
                    granted: true,
                    already_granted: None,
                    entry,
                    network_id: params.network_id,
                    full_granted_network_whitelist: change.full,
                }
            }

            Err(err) => {
                return Err(BridgeMethodHandleError::internal_error(format!(
                    "failed to grant requested network whitelist entry: {err}"
                )));
            }
        };

        bridge_result(result)
    }
}
