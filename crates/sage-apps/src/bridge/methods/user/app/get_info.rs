use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::bridge::methods::shared::{BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, BridgeMethodHandleError};
use crate::bridge::{RustBridgeRequest};
use crate::lifecycle::parse_network_permission_target;

#[derive(Debug, Clone, Copy)]
pub struct AppGetInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageNetworkPermissionInfo {
    pub scheme: String,
    pub host: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppGetInfoResult {
    pub id: String,
    pub name: String,
    pub version: String,
    pub requested_permissions: crate::types::SageRequestedPermissions,
    pub capabilities: Vec<UserBridgeCapability>,
    pub network: Vec<SageNetworkPermissionInfo>,
}

#[async_trait]
impl BridgeMethod for AppGetInfo {
    fn name(&self) -> &'static str {
        "app.getInfo"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::AppGetInfo)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        _tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let required_network = required_network_set(&ctx)?;

        let network = ctx
            .app
            .granted_permissions()
            .network
            .whitelist
            .iter()
            .map(|entry| SageNetworkPermissionInfo {
                scheme: entry.scheme.clone(),
                host: entry.host.clone(),
                required: required_network.contains(&(entry.scheme.clone(), entry.host.clone())),
            })
            .collect::<Vec<_>>();

        Ok(Box::new(AppGetInfoResult {
            id: ctx.app.id().to_string(),
            name: ctx.app.name().to_string(),
            version: ctx.app.version().to_string(),
            requested_permissions: ctx.app.requested_permissions().clone(),
            capabilities: UserBridgeCapability::shared_from_granted(&ctx.app.granted_permissions().capabilities),
            network,
        }))
    }
}

fn required_network_set(
    ctx: &BridgeContext<'_>,
) -> Result<BTreeSet<(String, String)>, BridgeMethodHandleError> {
    let mut out = BTreeSet::new();

    for entry in &ctx.app.requested_permissions().network.whitelist.required {
        let normalized = parse_network_permission_target(&format!(
            "{}://{}",
            entry.scheme, entry.host
        ))
            .map_err(BridgeMethodHandleError::internal_error)?;

        out.insert((normalized.scheme, normalized.host));
    }

    Ok(out)
}
