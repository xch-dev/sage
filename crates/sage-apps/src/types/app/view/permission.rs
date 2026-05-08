use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::capabilities::list::{SystemBridgeCapability, UserBridgeCapability};
use crate::capabilities::{CapabilityDefinition, CapabilityFlags};
use crate::types::app::view::network::SageNetworkWhitelistEntryView;
use crate::types::permissions::SageGrantedNetworkPermissions;
use crate::types::{
    SageGrantedPermissions, SageGrantedSystemPermissions, SageNetworkWhitelistEntry,
    SageRequestedPermissions,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityFlagsView {
    externally_observable: bool,
    accesses_sensitive_secret: bool,
    requestable_by_app: bool,
    user_grantable: bool,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
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
    #[serde(default)]
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,

    #[serde(default)]
    whitelist_by_network: BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>>,
}

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

    #[serde(default)]
    whitelist_by_network: BTreeMap<String, BTreeSet<SageNetworkWhitelistEntryView>>,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedSystemPermissionsView {
    capabilities: Vec<SystemBridgeCapability>,
}

impl SageGrantedPermissionsInput {
    pub fn new(
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        network_whitelist_by_network: BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>>,
    ) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            network: SageGrantedNetworkPermissionsInput {
                whitelist: network_whitelist.into_iter().collect(),
                whitelist_by_network: network_whitelist_by_network,
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
            self.network.whitelist_by_network.clone(),
        )
    }

    pub fn capabilities(&self) -> impl Iterator<Item = UserBridgeCapability> + '_ {
        self.capabilities.iter().copied()
    }

    pub fn network_whitelist(&self) -> impl Iterator<Item = SageNetworkWhitelistEntry> + '_ {
        self.network.whitelist.iter().cloned()
    }

    pub fn network_whitelist_by_network(
        &self,
    ) -> &BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>> {
        &self.network.whitelist_by_network
    }
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
            whitelist: value.whitelist().iter().map(Into::into).collect(),
            whitelist_by_network: value
                .whitelist_by_network()
                .iter()
                .map(|(network_id, entries)| {
                    (
                        network_id.clone(),
                        entries.iter().map(Into::into).collect(),
                    )
                })
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
