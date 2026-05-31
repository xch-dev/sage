use serde::Serialize;
use specta::Type;

use crate::{
    GrantedCapabilitiesChange, GrantedNetworkWhitelistChange, SageNetworkWhitelistEntry,
    UserBridgeCapability, UserRuntimeEvent,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedCapabilitiesChangeEvent {
    pub removed: Vec<UserBridgeCapability>,
    pub added: Vec<UserBridgeCapability>,
    pub full: Vec<UserBridgeCapability>,
}

impl GrantedCapabilitiesChangeEvent {
    pub fn from_change(change: &GrantedCapabilitiesChange) -> Self {
        Self {
            removed: change.removed.clone(),
            added: change.added.clone(),
            full: change.full.clone(),
        }
    }
}

impl UserRuntimeEvent for GrantedCapabilitiesChangeEvent {
    const TYPE: &'static str = "grantedCapabilitiesChange";
    const REQUIRED_CAPABILITY: UserBridgeCapability =
        UserBridgeCapability::AppRequestCapabilityGrant;
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GrantedNetworkWhitelistChangeEvent {
    pub removed: Vec<SageNetworkWhitelistEntry>,
    pub added: Vec<SageNetworkWhitelistEntry>,
    pub full: Vec<SageNetworkWhitelistEntry>,
}

impl GrantedNetworkWhitelistChangeEvent {
    pub fn from_change(change: &GrantedNetworkWhitelistChange) -> Self {
        Self {
            removed: change.removed.clone(),
            added: change.added.clone(),
            full: change.full.clone(),
        }
    }
}

impl UserRuntimeEvent for GrantedNetworkWhitelistChangeEvent {
    const TYPE: &'static str = "grantedNetworkWhitelistChange";
    const REQUIRED_CAPABILITY: UserBridgeCapability =
        UserBridgeCapability::AppRequestNetworkWhitelistGrant;
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
    const REQUIRED_CAPABILITY: UserBridgeCapability =
        UserBridgeCapability::AppLifecycleSetBeforeStopListener;
}
