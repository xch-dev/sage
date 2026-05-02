use crate::capabilities::list::UserBridgeCapability;
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct AppUpdateResult {
    change: GrantedPermissionsChange,
}

#[derive(Debug)]
pub struct GrantedPermissionsChange {
    network_whitelist: GrantedNetworkWhitelistChange,
    capabilities: GrantedCapabilitiesChange,
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
    pub fn from_update(entry: &SageNetworkWhitelistEntry, update_result: &AppUpdateResult) -> Self {
        Self::from_change(entry, update_result.change().network_whitelist())
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
        }
    }

    pub fn capabilities(&self) -> &GrantedCapabilitiesChange {
        &self.capabilities
    }

    pub fn network_whitelist(&self) -> &GrantedNetworkWhitelistChange {
        &self.network_whitelist
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
