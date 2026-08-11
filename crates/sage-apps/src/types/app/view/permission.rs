use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    CapabilityDefinition, CapabilityFlags, SageGrantedNetworkPermissions, SageGrantedPermissions,
    SageGrantedSystemPermissions, SageNetworkWhitelistEntry, SageNetworkWhitelistEntryView,
    SageRequestedPermissions, SystemBridgeCapability, UserBridgeCapability,
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

    pub fn with_additional(mut self, additional: SageGrantedPermissionsInput) -> Self {
        self.capabilities.extend(additional.capabilities);

        self.network.whitelist.extend(additional.network.whitelist);

        for (network_id, entries) in additional.network.whitelist_by_network {
            self.network
                .whitelist_by_network
                .entry(network_id)
                .or_default()
                .extend(entries);
        }

        self
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
                    (network_id.clone(), entries.iter().map(Into::into).collect())
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

impl From<(&SageGrantedPermissions, &SageRequestedPermissions)> for SageGrantedPermissionsInput {
    fn from((granted, requested): (&SageGrantedPermissions, &SageRequestedPermissions)) -> Self {
        let capabilities = granted
            .capabilities()
            .copied()
            .filter(|capability| requested.capabilities().is_allowed(*capability));

        let network_whitelist = granted
            .network()
            .whitelist_iter()
            .filter(|entry| requested.network().whitelist().is_allowed(entry))
            .cloned()
            .chain(requested.network().whitelist().required().cloned());

        let mut whitelist_by_network = granted
            .network()
            .whitelist_by_network()
            .iter()
            .filter_map(|(network_id, granted_entries)| {
                let requested_whitelist =
                    requested.network().whitelist_by_network().get(network_id)?;

                let entries = granted_entries
                    .iter()
                    .filter(|entry| requested_whitelist.is_allowed(entry))
                    .cloned()
                    .collect::<BTreeSet<_>>();

                if entries.is_empty() {
                    None
                } else {
                    Some((network_id.clone(), entries))
                }
            })
            .collect::<BTreeMap<_, _>>();

        for (network_id, requested_whitelist) in requested.network().whitelist_by_network() {
            let required = requested_whitelist.required().cloned().collect::<Vec<_>>();

            if !required.is_empty() {
                whitelist_by_network
                    .entry(network_id.clone())
                    .or_default()
                    .extend(required);
            }
        }

        Self::new(capabilities, network_whitelist, whitelist_by_network)
    }
}
