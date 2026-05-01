use crate::bridge::capabilities::UserBridgeCapability;
use crate::lifecycle::update::types::{GrantedCapabilitiesChange, GrantedNetworkWhitelistChange};
use crate::types::SageNetworkWhitelistEntry;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum EventForApp {
    GrantedCapabilitiesChange(GrantedCapabilitiesChangeEvent),
    GrantedNetworkWhitelistChange(GrantedNetworkWhitelistChangeEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedCapabilitiesChangeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub removed_granted_capabilities: Vec<UserBridgeCapability>,
    pub added_granted_capabilities: Vec<UserBridgeCapability>,
    pub full_granted_capabilities: Vec<UserBridgeCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedNetworkWhitelistChangeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub removed_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    pub added_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    pub full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
}

impl EventForApp {
    pub fn from_capabilities_change(change: &GrantedCapabilitiesChange) -> Self {
        EventForApp::GrantedCapabilitiesChange(GrantedCapabilitiesChangeEvent {
            event_type: "grantedCapabilitiesChange".to_string(),
            removed_granted_capabilities: change.removed.clone(),
            added_granted_capabilities: change.added.clone(),
            full_granted_capabilities: change.full.clone(),
        })
    }

    pub fn from_network_whitelist_change(
        change: &GrantedNetworkWhitelistChange,
    ) -> Self {
        EventForApp::GrantedNetworkWhitelistChange(GrantedNetworkWhitelistChangeEvent {
            event_type: "grantedNetworkWhitelistChange".to_string(),
            removed_granted_network_whitelist: change.removed.clone(),
            added_granted_network_whitelist: change.added.clone(),
            full_granted_network_whitelist: change.full.clone(),
        })
    }
}
