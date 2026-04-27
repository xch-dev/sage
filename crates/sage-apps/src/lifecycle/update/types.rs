use crate::bridge::capabilities::UserBridgeCapability;
use crate::types::SageNetworkWhitelistEntry;

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
