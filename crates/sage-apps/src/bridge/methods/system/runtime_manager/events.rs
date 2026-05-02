use serde::Serialize;
use specta::Type;

use crate::bridge::event_emit::{AppRuntimeEvent, AppRuntimeEventRail};
use crate::runtime::SageAppRuntimeRecordView;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeManagerRuntimesChangedEvent {
    pub runtimes: Vec<SageAppRuntimeRecordView>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActiveRuntimeChangedEvent {
    pub host_window_label: String,
    pub app_id: Option<String>,
    pub runtime_id: Option<String>,
}

impl RuntimeManagerRuntimesChangedEvent {
    pub(crate) fn new(runtimes: Vec<SageAppRuntimeRecordView>) -> Self {
        Self { runtimes }
    }
}

impl AppRuntimeEvent for RuntimeManagerRuntimesChangedEvent {
    const TYPE: &'static str = "runtimeManager.runtimesChanged";
    const RAIL: AppRuntimeEventRail = AppRuntimeEventRail::System;
}

impl AppRuntimeEvent for ActiveRuntimeChangedEvent {
    const TYPE: &'static str = "runtimeManager.activeRuntimeChanged";
    const RAIL: AppRuntimeEventRail = AppRuntimeEventRail::System;
}
