use crate::runtime::SageAppRuntimeRecordView;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeManagerRuntimesChangedEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub runtimes: Vec<SageAppRuntimeRecordView>,
}

impl RuntimeManagerRuntimesChangedEvent {
    pub fn new(runtimes: Vec<SageAppRuntimeRecordView>) -> Self {
        Self {
            event_type: "runtimeManager.runtimesChanged".to_string(),
            runtimes,
        }
    }
}
