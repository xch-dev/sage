use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeTools, RustBridgeRequest, SageGrantedNetworkPermissions,
    SageRequestedNetworkPermissions, SageRequestedPermissions, UserBridgeCapability,
};

#[derive(Debug, Clone, Copy)]
pub struct AppGetInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageNetworkPermissionInfo {
    pub scheme: String,
    pub host: String,
    pub required: bool,
}

fn effective_network_permissions(
    granted: &SageGrantedNetworkPermissions,
    requested: &SageRequestedNetworkPermissions,
    network_id: &str,
) -> Vec<SageNetworkPermissionInfo> {
    let requested_for_network = requested.whitelist_by_network().get(network_id);

    granted
        .effective_whitelist_for_network(network_id)
        .into_iter()
        .map(|entry| SageNetworkPermissionInfo {
            scheme: entry.scheme().to_string(),
            host: entry.host().to_string(),
            required: requested.whitelist().is_required(&entry)
                || requested_for_network.is_some_and(|whitelist| whitelist.is_required(&entry)),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppGetInfoResult {
    pub id: String,
    pub name: String,
    pub version: String,
    pub requested_permissions: SageRequestedPermissions,
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
        tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let network_id = tools.app_state.lock().await.network_id();

        let result = ctx.app.with(|app| {
            let network = effective_network_permissions(
                app.granted_permissions().network(),
                app.requested_permissions().network(),
                &network_id,
            );

            AppGetInfoResult {
                id: app.id().to_string(),
                name: app.name().to_string(),
                version: app.version().to_string(),
                requested_permissions: app.requested_permissions().clone(),
                capabilities: app.granted_permissions().shared_capabilities(),
                network,
            }
        });

        Ok(Box::new(result))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::{SageNetworkWhitelistEntry, SageRequestedNetworkWhitelist};

    fn entry(host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new("https", host).unwrap()
    }

    #[test]
    fn app_info_network_permissions_include_active_network_grants() {
        let shared_required = entry("shared-required.example.com");
        let shared_optional = entry("shared-optional.example.com");
        let mainnet_required = entry("mainnet.example.com");
        let testnet_required = entry("testnet.example.com");

        let requested = SageRequestedNetworkPermissions::new(
            [shared_required.clone()],
            [shared_optional.clone()],
            [
                (
                    "mainnet".to_string(),
                    SageRequestedNetworkWhitelist::new([mainnet_required.clone()], []),
                ),
                (
                    "testnet11".to_string(),
                    SageRequestedNetworkWhitelist::new([testnet_required.clone()], []),
                ),
            ],
        )
        .unwrap();

        let granted = SageGrantedNetworkPermissions::new(
            &requested,
            [shared_required, shared_optional],
            BTreeMap::from([
                ("mainnet".to_string(), BTreeSet::from([mainnet_required])),
                ("testnet11".to_string(), BTreeSet::from([testnet_required])),
            ]),
        )
        .unwrap();

        assert_eq!(
            effective_network_permissions(&granted, &requested, "mainnet"),
            vec![
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "mainnet.example.com".to_string(),
                    required: true,
                },
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "shared-optional.example.com".to_string(),
                    required: false,
                },
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "shared-required.example.com".to_string(),
                    required: true,
                },
            ]
        );

        assert_eq!(
            effective_network_permissions(&granted, &requested, "testnet11"),
            vec![
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "shared-optional.example.com".to_string(),
                    required: false,
                },
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "shared-required.example.com".to_string(),
                    required: true,
                },
                SageNetworkPermissionInfo {
                    scheme: "https".to_string(),
                    host: "testnet.example.com".to_string(),
                    required: true,
                },
            ]
        );
    }
}
