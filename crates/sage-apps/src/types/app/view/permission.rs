use std::collections::BTreeSet;
use serde::{Serialize};
use specta::Type;
use crate::bridge::capabilities::{SystemBridgeCapability, UserBridgeCapability};
use crate::types::{SageGrantedPermissions, SageGrantedSystemPermissions};
use crate::types::app::view::network::SageNetworkWhitelistEntryView;
use crate::types::permissions::SageGrantedNetworkPermissions;

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedPermissionsView {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissionsView,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedNetworkPermissionsView {
    whitelist: BTreeSet<SageNetworkWhitelistEntryView>,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
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
            whitelist: value
                .whitelist()
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<&SageGrantedSystemPermissions> for SageGrantedSystemPermissionsView {
    fn from(value: &SageGrantedSystemPermissions) -> Self {
        Self {
            capabilities: value.capabilities().to_vec(),
        }
    }
}
