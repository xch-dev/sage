use serde::Serialize;
use specta::Type;

use crate::capabilities::list::UserBridgeCapability;
use crate::bridge::event_emit::UserRuntimeEvent;
use crate::lifecycle::update::types::{GrantedCapabilitiesChange, GrantedNetworkWhitelistChange};
use crate::types::SageNetworkWhitelistEntry;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedCapabilitiesChangeEvent {
    pub removed_granted_capabilities: Vec<UserBridgeCapability>,
    pub added_granted_capabilities: Vec<UserBridgeCapability>,
    pub full_granted_capabilities: Vec<UserBridgeCapability>,
}

impl GrantedCapabilitiesChangeEvent {
    pub fn from_change(change: &GrantedCapabilitiesChange) -> Self {
        Self {
            removed_granted_capabilities: change.removed.clone(),
            added_granted_capabilities: change.added.clone(),
            full_granted_capabilities: change.full.clone(),
        }
    }
}

impl UserRuntimeEvent for GrantedCapabilitiesChangeEvent {
    const TYPE: &'static str = "grantedCapabilitiesChange";
    const REQUIRED_CAPABILITY: UserBridgeCapability = UserBridgeCapability::AppRequestCapabilityGrant;
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedNetworkWhitelistChangeEvent {
    pub removed_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    pub added_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
    pub full_granted_network_whitelist: Vec<SageNetworkWhitelistEntry>,
}

impl GrantedNetworkWhitelistChangeEvent {
    pub fn from_change(change: &GrantedNetworkWhitelistChange) -> Self {
        Self {
            removed_granted_network_whitelist: change.removed.clone(),
            added_granted_network_whitelist: change.added.clone(),
            full_granted_network_whitelist: change.full.clone(),
        }
    }
}

impl UserRuntimeEvent for GrantedNetworkWhitelistChangeEvent {
    const TYPE: &'static str = "grantedNetworkWhitelistChange";
    const REQUIRED_CAPABILITY: UserBridgeCapability = UserBridgeCapability::AppRequestNetworkWhitelistGrant;
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BeforeStopEvent {
    pub request_id: String,
}

impl BeforeStopEvent {
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
        }
    }
}

impl UserRuntimeEvent for BeforeStopEvent {
    const TYPE: &'static str = "lifecycle.beforeStop";
    const REQUIRED_CAPABILITY: UserBridgeCapability = UserBridgeCapability::AppLifecycleSetBeforeStopListener;
}
