use crate::bridge::capabilities::UserBridgeCapability;
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry, UserSageApp};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct AppUpdateResult {
    app: UserSageApp,
    change: GrantedPermissionsChange,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    pub fn new(app: UserSageApp, change: GrantedPermissionsChange) -> Self {
        Self { app, change }
    }

    pub fn app(&self) -> &UserSageApp {
        &self.app
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
}
