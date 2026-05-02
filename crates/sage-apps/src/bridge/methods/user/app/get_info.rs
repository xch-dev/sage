use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::RustBridgeRequest;
use crate::capabilities::list::UserBridgeCapability;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};

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
        let result = ctx
            .app
            .with(|app| {
                let network = app
                    .granted_permissions()
                    .network()
                    .whitelist_iter()
                    .map(|entry| SageNetworkPermissionInfo {
                        scheme: entry.scheme().to_string(),
                        host: entry.host().to_string(),
                        required: app
                            .requested_permissions()
                            .network()
                            .whitelist()
                            .is_required(entry),
                    })
                    .collect::<Vec<_>>();
                AppGetInfoResult {
                    id: app.id().to_string(),
                    name: app.name().to_string(),
                    version: app.version().to_string(),
                    requested_permissions: app.requested_permissions().clone(),
                    capabilities: app.granted_permissions().shared_capabilities(),
                    network,
                }
            }
        );

        Ok(Box::new(result))
    }
}
