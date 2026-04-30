use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::capabilities::{CapabilityDefinition, CapabilityFlags};
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedPermissions};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityFlagsView {
    externally_observable: bool,
    accesses_sensitive_secret: bool,
    requestable_by_app: bool,
    user_grantable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityDefinitionView {
    key: String,
    label: String,
    description: String,
    flags: SageAppCapabilityFlagsView,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedPermissionsInput {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissionsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedNetworkPermissionsInput {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedPermissionsView {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissionsView,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedNetworkPermissionsView {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

impl SageGrantedPermissionsInput {
    pub fn new(
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            network: SageGrantedNetworkPermissionsInput {
                whitelist: network_whitelist.into_iter().collect(),
            },
        }
    }

    pub fn resolve(
        &self,
        requested: &SageRequestedPermissions,
    ) -> anyhow::Result<SageGrantedPermissions> {
        SageGrantedPermissions::new(
            requested,
            self.capabilities.iter().copied(),
            self.network.whitelist.iter().cloned(),
        )
    }

    pub fn capabilities(&self) -> impl Iterator<Item = UserBridgeCapability> + '_ {
        self.capabilities.iter().copied()
    }

    pub fn network_whitelist(&self) -> impl Iterator<Item = SageNetworkWhitelistEntry> + '_ {
        self.network.whitelist.iter().cloned()
    }
}

impl From<&SageGrantedPermissions> for SageGrantedPermissionsView {
    fn from(permissions: &SageGrantedPermissions) -> Self {
        Self {
            capabilities: permissions.shared_capabilities().into_iter().collect(),
            network: SageGrantedNetworkPermissionsView {
                whitelist: permissions.network().whitelist().clone(),
            },
        }
    }
}

impl From<CapabilityFlags> for SageAppCapabilityFlagsView {
    fn from(flags: CapabilityFlags) -> Self {
        Self {
            externally_observable: flags.externally_observable(),
            accesses_sensitive_secret: flags.accesses_sensitive_secret(),
            requestable_by_app: flags.requestable_by_app(),
            user_grantable: flags.user_grantable(),
        }
    }
}

impl From<CapabilityDefinition<UserBridgeCapability>> for SageAppCapabilityDefinitionView {
    fn from(definition: CapabilityDefinition<UserBridgeCapability>) -> Self {
        Self {
            key: definition.capability().key().to_string(),
            label: definition.label().to_string(),
            description: definition.description().to_string(),
            flags: definition.flags().into(),
        }
    }
}
