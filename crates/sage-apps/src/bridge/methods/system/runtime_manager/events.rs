use crate::runtime::SageAppRuntimeRecordView;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeManagerRuntimesChangedEvent {
    pub channel: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub runtimes: Vec<SageAppRuntimeRecordView>,
}

impl RuntimeManagerRuntimesChangedEvent {
    pub fn new(channel: String, runtimes: Vec<SageAppRuntimeRecordView>) -> Self {
        Self {
            channel,
            event_type: "runtimeManager.runtimesChanged".to_string(),
            runtimes,
        }
    }
}
