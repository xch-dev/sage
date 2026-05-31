use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

use super::{SageRequestedNetworkPermissions, SageRequestedPermissions};
use crate::{build_user_grantable_capability_set, get_user_capability_definition, SageNetworkWhitelistEntry, SharedCapabilitiesExt, SystemBridgeCapability, UserBridgeCapability, validate_permissions_policy};

pub type NetworkWhitelistByNetwork = BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>>;

pub fn network_whitelist_by_network_from_iter<I, E>(items: I) -> NetworkWhitelistByNetwork
where
    I: IntoIterator<Item = (String, E)>,
    E: IntoIterator<Item = SageNetworkWhitelistEntry>,
{
    items
        .into_iter()
        .map(|(network_id, entries)| (network_id, entries.into_iter().collect::<BTreeSet<_>>()))
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedNetworkPermissions {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
    whitelist_by_network: NetworkWhitelistByNetwork,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct SageGrantedPermissions {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Type)]
pub struct SageGrantedSystemPermissions {
    capabilities: Vec<SystemBridgeCapability>,
}

impl SageGrantedSystemPermissions {
    pub fn new(capabilities: impl IntoIterator<Item = SystemBridgeCapability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    pub fn capabilities(&self) -> &[SystemBridgeCapability] {
        &self.capabilities
    }
}

impl SageGrantedNetworkPermissions {
    pub fn new(
        requested: &SageRequestedNetworkPermissions,
        whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        whitelist_by_network: NetworkWhitelistByNetwork,
    ) -> anyhow::Result<Self> {
        let whitelist = whitelist.into_iter().collect::<BTreeSet<_>>();

        for entry in &whitelist {
            if !requested.whitelist().is_allowed(entry) {
                anyhow::bail!(
                    "granted shared network whitelist entry not requested in manifest: {}",
                    entry.as_permission_string()
                );
            }
        }

        let mut by_network = BTreeMap::new();

        for (network_id, entries) in whitelist_by_network {
            let network_id = network_id.trim().to_string();

            if network_id.is_empty() {
                anyhow::bail!("granted network whitelist network id cannot be empty");
            }

            let Some(requested_whitelist) = requested.whitelist_by_network().get(&network_id)
            else {
                anyhow::bail!(
                    "granted network-specific whitelist entry for unrequested network: {network_id}",
                );
            };

            for entry in &entries {
                if !requested_whitelist.is_allowed(entry) {
                    anyhow::bail!(
                        "granted network-specific whitelist entry not requested in manifest for {}: {}",
                        network_id,
                        entry.as_permission_string()
                    );
                }
            }

            by_network.insert(network_id, entries);
        }

        Ok(Self {
            whitelist,
            whitelist_by_network: by_network,
        })
    }

    pub fn whitelist(&self) -> &BTreeSet<SageNetworkWhitelistEntry> {
        &self.whitelist
    }

    pub fn whitelist_iter(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.whitelist.iter()
    }

    pub fn whitelist_by_network(&self) -> &NetworkWhitelistByNetwork {
        &self.whitelist_by_network
    }

    pub fn effective_whitelist_for_network(
        &self,
        network_id: &str,
    ) -> Vec<SageNetworkWhitelistEntry> {
        self.whitelist
            .iter()
            .cloned()
            .chain(
                self.whitelist_by_network
                    .get(network_id)
                    .into_iter()
                    .flat_map(|entries| entries.iter().cloned()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn all_whitelist_entries(&self) -> Vec<SageNetworkWhitelistEntry> {
        self.whitelist
            .iter()
            .cloned()
            .chain(
                self.whitelist_by_network
                    .values()
                    .flat_map(|entries| entries.iter().cloned()),
            )
            .collect()
    }
}

impl SageGrantedPermissions {
    pub fn new(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        network_whitelist_by_network: NetworkWhitelistByNetwork,
    ) -> anyhow::Result<Self> {
        Self::new_with_extra_granted_capabilities(
            requested,
            capabilities,
            std::iter::empty(),
            network_whitelist,
            network_whitelist_by_network,
        )
    }

    pub(crate) fn new_with_extra_granted_capabilities(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        extra_granted_capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        network_whitelist_by_network: NetworkWhitelistByNetwork,
    ) -> anyhow::Result<Self> {
        let mut capabilities =
            build_user_grantable_capability_set(requested.capabilities(), capabilities)?;

        for capability in extra_granted_capabilities {
            let definition = get_user_capability_definition(capability);

            if !definition.flags().user_grantable() {
                anyhow::bail!(
                    "extra granted capability is not user grantable: {}",
                    capability.key()
                );
            }

            if definition.flags().requestable_by_app() {
                anyhow::bail!(
                    "extra granted capability must not be app-manifest requestable: {}",
                    capability.key()
                );
            }

            capabilities.insert(capability);
        }

        let network = SageGrantedNetworkPermissions::new(
            requested.network(),
            network_whitelist,
            network_whitelist_by_network,
        )?;

        let effective_capabilities = requested
            .capabilities()
            .resolve_effective_grants(capabilities.iter().copied());

        validate_permissions_policy(
            effective_capabilities,
            network.all_whitelist_entries(),
            "granted permissions",
        )?;

        Ok(Self {
            capabilities,
            network,
        })
    }

    pub fn from_requested_and_granted(
        requested: &SageRequestedPermissions,
        granted: SageGrantedPermissions,
    ) -> anyhow::Result<Self> {
        Self::new(
            requested,
            granted.capabilities,
            granted.network.whitelist,
            granted.network.whitelist_by_network,
        )
    }

    pub fn with_capability_added(
        &self,
        requested: &SageRequestedPermissions,
        capability: UserBridgeCapability,
    ) -> anyhow::Result<Self> {
        Self::new(
            requested,
            self.capabilities.iter().copied().chain([capability]),
            self.network.whitelist_iter().cloned(),
            self.network.whitelist_by_network().clone(),
        )
    }

    pub fn with_network_whitelist_entry_added(
        &self,
        requested: &SageRequestedPermissions,
        entry: SageNetworkWhitelistEntry,
    ) -> anyhow::Result<Self> {
        Self::new(
            requested,
            self.capabilities.iter().copied(),
            self.network.whitelist_iter().cloned().chain([entry]),
            self.network.whitelist_by_network().clone(),
        )
    }

    pub fn with_network_whitelist_entry_for_network_added(
        &self,
        requested: &SageRequestedPermissions,
        network_id: impl Into<String>,
        entry: SageNetworkWhitelistEntry,
    ) -> anyhow::Result<Self> {
        let network_id = network_id.into();
        let mut by_network = self.network.whitelist_by_network().clone();

        by_network.entry(network_id).or_default().insert(entry);

        Self::new(
            requested,
            self.capabilities.iter().copied(),
            self.network.whitelist_iter().cloned(),
            by_network,
        )
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.capabilities.iter()
    }

    pub fn has_capability(&self, capability: UserBridgeCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn capabilities_vec(&self) -> Vec<UserBridgeCapability> {
        self.capabilities.iter().copied().collect()
    }

    pub fn network(&self) -> &SageGrantedNetworkPermissions {
        &self.network
    }

    pub fn network_whitelist_vec(&self) -> Vec<SageNetworkWhitelistEntry> {
        self.network.whitelist_iter().cloned().collect()
    }

    pub fn network_whitelist_by_network(&self) -> &NetworkWhitelistByNetwork {
        self.network.whitelist_by_network()
    }

    pub fn shared_capabilities(&self) -> Vec<UserBridgeCapability> {
        self.capabilities().copied().shared()
    }

    pub fn for_builtin_requested(requested: &SageRequestedPermissions) -> anyhow::Result<Self> {
        let required_by_network = network_whitelist_by_network_from_iter(
            requested
                .network()
                .whitelist_by_network()
                .iter()
                .map(|(network_id, whitelist)| {
                    (
                        network_id.clone(),
                        whitelist.required().cloned().collect::<Vec<_>>(),
                    )
                }),
        );

        Self::new(
            requested,
            requested.capabilities().user_grantable(),
            requested.network().whitelist().required().cloned(),
            required_by_network,
        )
    }

    #[cfg(test)]
    pub fn new_unchecked(
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        network_whitelist_by_network: NetworkWhitelistByNetwork,
    ) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            network: SageGrantedNetworkPermissions {
                whitelist: network_whitelist.into_iter().collect(),
                whitelist_by_network: network_whitelist_by_network,
            },
        }
    }
}

impl<'de> Deserialize<'de> for SageGrantedPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawSageGrantedPermissions {
            #[serde(default)]
            capabilities: BTreeSet<UserBridgeCapability>,

            #[serde(default)]
            network: SageGrantedNetworkPermissions,
        }

        let raw = RawSageGrantedPermissions::deserialize(deserializer)?;

        validate_permissions_policy(
            raw.capabilities.iter().copied(),
            raw.network.all_whitelist_entries(),
            "granted permissions",
        )
        .map_err(serde::de::Error::custom)?;

        Ok(Self {
            capabilities: raw.capabilities,
            network: raw.network,
        })
    }
}

impl<'de> Deserialize<'de> for SageGrantedNetworkPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            #[serde(default)]
            whitelist: BTreeSet<SageNetworkWhitelistEntry>,

            #[serde(default)]
            whitelist_by_network: NetworkWhitelistByNetwork,
        }

        let raw = Raw::deserialize(deserializer)?;

        Ok(Self {
            whitelist: raw.whitelist,
            whitelist_by_network: raw.whitelist_by_network,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions};

    #[test]
    fn granted_permissions_reject_unrequested_shared_network_whitelist_entry() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [SageNetworkWhitelistEntry::new_unchecked(
                    "https",
                    "api.example.com",
                )],
                [],
                [],
            )
            .unwrap(),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::StoragePersistentWebview],
                [UserBridgeCapability::WalletSendXch],
            ),
        )
        .unwrap();

        let err = SageGrantedPermissions::new(
            &requested,
            [UserBridgeCapability::StoragePersistentWebview],
            [SageNetworkWhitelistEntry::new_unchecked(
                "https",
                "evil.example.com",
            )],
            BTreeMap::new(),
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("granted shared network whitelist entry not requested")
        );
    }
}
