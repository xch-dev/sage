use serde::Serialize;
use specta::Type;

use crate::bridge::event_emit::SystemRuntimeEvent;
use crate::capabilities::list::SystemBridgeCapability;
use crate::runtime::SageAppRuntimeRecordView;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeManagerRuntimesChangedEvent {
    pub runtimes: Vec<SageAppRuntimeRecordView>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeManagerActiveTaskbarRuntimeChangedEvent {
    pub host_window_label: String,
    pub app_id: Option<String>,
    pub runtime_id: Option<String>,
}

impl RuntimeManagerRuntimesChangedEvent {
    pub(crate) fn new(runtimes: Vec<SageAppRuntimeRecordView>) -> Self {
        Self { runtimes }
    }
}

impl SystemRuntimeEvent for RuntimeManagerRuntimesChangedEvent {
    const TYPE: &'static str = "runtimeManager.runtimesChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability =
        SystemBridgeCapability::RuntimeManagerListenRuntimesChanged;
}

impl SystemRuntimeEvent for RuntimeManagerActiveTaskbarRuntimeChangedEvent {
    const TYPE: &'static str = "runtimeManager.activeTaskbarRuntimeChanged";
    const REQUIRED_CAPABILITY: SystemBridgeCapability =
        SystemBridgeCapability::RuntimeManagerListenActiveTaskbarRuntimeChanged;
}
