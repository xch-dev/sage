use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::bridge::capabilities::{SystemBridgeCapability, UserBridgeCapability};
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry};
use crate::types::permissions::SageGrantedNetworkPermissions;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedPermissionsView {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissionsView,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedNetworkPermissionsView {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedSystemPermissionsView {
    capabilities: Vec<SystemBridgeCapability>,
}

impl From<&SageGrantedPermissions> for SageGrantedPermissionsView {
    fn from(permissions: &SageGrantedPermissions) -> Self {
        Self {
            network: permissions.network().into(),
            capabilities: permissions.shared_capabilities().iter().copied().collect(),
        }
    }
}

impl From<&SageGrantedNetworkPermissions> for SageGrantedNetworkPermissionsView {
    fn from(value: &SageGrantedNetworkPermissions) -> Self {
        Self {
            whitelist: value.whitelist().clone()
        }
    }
}
