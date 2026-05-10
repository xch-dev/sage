use crate::capabilities::list::UserBridgeCapability;
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct AppUpdateResult {
    change: GrantedPermissionsChange,
}

#[derive(Debug)]
pub struct GrantedPermissionsChange {
    capabilities: GrantedCapabilitiesChange,
    network_whitelist: GrantedNetworkWhitelistChange,
    network_whitelist_by_network: GrantedNetworkWhitelistByNetworkChange,
}

#[derive(Debug, Clone)]
pub struct GrantedCapabilitiesChange {
    pub removed: Vec<UserBridgeCapability>,
    pub added: Vec<UserBridgeCapability>,
    pub full: Vec<UserBridgeCapability>,
}

#[derive(Debug, Clone)]
pub struct GrantedNetworkWhitelistChange {
    pub removed: Vec<SageNetworkWhitelistEntry>,
    pub added: Vec<SageNetworkWhitelistEntry>,
    pub full: Vec<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone)]
pub struct GrantedNetworkWhitelistByNetworkChange {
    pub removed: BTreeMap<String, Vec<SageNetworkWhitelistEntry>>,
    pub added: BTreeMap<String, Vec<SageNetworkWhitelistEntry>>,
    pub full: BTreeMap<String, Vec<SageNetworkWhitelistEntry>>,
}

#[derive(Debug)]
pub enum GrantCapabilityOutcome {
    AlreadyGranted {
        capability: UserBridgeCapability,
        full_granted_capabilities: Vec<UserBridgeCapability>,
    },
    Granted {
        capability: UserBridgeCapability,
        change: GrantedCapabilitiesChange,
    },
}

impl GrantCapabilityOutcome {
    pub fn from_update(capability: UserBridgeCapability, update_result: &AppUpdateResult) -> Self {
        Self::from_change(capability, update_result.change().capabilities())
    }
    fn from_change(capability: UserBridgeCapability, change: &GrantedCapabilitiesChange) -> Self {
        if change.added.is_empty() && change.removed.is_empty() {
            Self::AlreadyGranted {
                capability,
                full_granted_capabilities: change.full.clone(),
            }
        } else {
            Self::Granted {
                capability,
                change: change.clone(),
            }
        }
    }
}

#[derive(Debug)]
pub enum GrantNetworkWhitelistOutcome {
    AlreadyGranted {
        entry: SageNetworkWhitelistEntry,
        full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    },
    Granted {
        entry: SageNetworkWhitelistEntry,
        change: GrantedNetworkWhitelistChange,
    },
}

impl GrantNetworkWhitelistOutcome {
    pub fn from_update(
        network_id: Option<&str>,
        entry: &SageNetworkWhitelistEntry,
        update_result: &AppUpdateResult,
    ) -> Self {
        let change = match network_id {
            Some(network_id) => update_result
                .change()
                .network_whitelist_by_network()
                .for_network(network_id),
            None => update_result.change().network_whitelist().clone(),
        };

        Self::from_change(entry, &change)
    }

    fn from_change(
        entry: &SageNetworkWhitelistEntry,
        change: &GrantedNetworkWhitelistChange,
    ) -> Self {
        if change.added.is_empty() && change.removed.is_empty() {
            Self::AlreadyGranted {
                entry: entry.clone(),
                full_granted_network_whitelist: change.full.clone(),
            }
        } else {
            Self::Granted {
                entry: entry.clone(),
                change: change.clone(),
            }
        }
    }
}

impl GrantedPermissionsChange {
    pub fn diff(previous: &SageGrantedPermissions, next: &SageGrantedPermissions) -> Self {
        Self {
            capabilities: GrantedCapabilitiesChange::diff(
                &previous.capabilities_vec(),
                &next.capabilities_vec(),
            ),
            network_whitelist: GrantedNetworkWhitelistChange::diff(
                &previous.network_whitelist_vec(),
                &next.network_whitelist_vec(),
            ),
            network_whitelist_by_network: GrantedNetworkWhitelistByNetworkChange::diff(
                previous.network().whitelist_by_network(),
                next.network().whitelist_by_network(),
            ),
        }
    }

    pub fn network_changed(&self) -> bool {
        !self.network_whitelist.is_empty() || !self.network_whitelist_by_network.is_empty()
    }

    pub fn capabilities(&self) -> &GrantedCapabilitiesChange {
        &self.capabilities
    }

    pub fn network_whitelist(&self) -> &GrantedNetworkWhitelistChange {
        &self.network_whitelist
    }

    pub fn network_whitelist_by_network(&self) -> &GrantedNetworkWhitelistByNetworkChange {
        &self.network_whitelist_by_network
    }
}

impl AppUpdateResult {
    pub fn new(change: GrantedPermissionsChange) -> Self {
        Self { change }
    }

    pub fn change(&self) -> &GrantedPermissionsChange {
        &self.change
    }
}

impl GrantedCapabilitiesChange {
    pub fn diff(previous: &[UserBridgeCapability], next: &[UserBridgeCapability]) -> Self {
        let previous_set: BTreeSet<_> = previous.iter().copied().collect();
        let next_set: BTreeSet<_> = next.iter().copied().collect();

        Self {
            removed: previous_set.difference(&next_set).copied().collect(),
            added: next_set.difference(&previous_set).copied().collect(),
            full: next.to_vec(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.added.is_empty()
    }
}

impl GrantedNetworkWhitelistChange {
    pub fn diff(
        previous: &[SageNetworkWhitelistEntry],
        next: &[SageNetworkWhitelistEntry],
    ) -> Self {
        let previous_set: BTreeSet<_> = previous.iter().cloned().collect();
        let next_set: BTreeSet<_> = next.iter().cloned().collect();

        Self {
            removed: previous_set.difference(&next_set).cloned().collect(),
            added: next_set.difference(&previous_set).cloned().collect(),
            full: next.to_vec(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.added.is_empty()
    }
}

impl GrantedNetworkWhitelistByNetworkChange {
    pub fn diff(
        previous: &BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>>,
        next: &BTreeMap<String, BTreeSet<SageNetworkWhitelistEntry>>,
    ) -> Self {
        let network_ids = previous
            .keys()
            .chain(next.keys())
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut removed = BTreeMap::new();
        let mut added = BTreeMap::new();
        let mut full = BTreeMap::new();

        for network_id in network_ids {
            let previous_entries = previous.get(&network_id).cloned().unwrap_or_default();

            let next_entries = next.get(&network_id).cloned().unwrap_or_default();

            let removed_entries = previous_entries
                .difference(&next_entries)
                .cloned()
                .collect::<Vec<_>>();

            let added_entries = next_entries
                .difference(&previous_entries)
                .cloned()
                .collect::<Vec<_>>();

            if !removed_entries.is_empty() {
                removed.insert(network_id.clone(), removed_entries);
            }

            if !added_entries.is_empty() {
                added.insert(network_id.clone(), added_entries);
            }

            if !next_entries.is_empty() {
                full.insert(network_id, next_entries.into_iter().collect::<Vec<_>>());
            }
        }

        Self {
            removed,
            added,
            full,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.added.is_empty()
    }

    pub fn for_network(&self, network_id: &str) -> GrantedNetworkWhitelistChange {
        GrantedNetworkWhitelistChange {
            removed: self.removed.get(network_id).cloned().unwrap_or_default(),
            added: self.added.get(network_id).cloned().unwrap_or_default(),
            full: self.full.get(network_id).cloned().unwrap_or_default(),
        }
    }
}
