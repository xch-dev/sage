use serde::Serialize;
use specta::Type;

use crate::bridge::event_emit::{AppRuntimeEvent, AppRuntimeEventRail};
use crate::runtime::SageAppRuntimeRecordView;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeManagerRuntimesChangedEvent {
    pub runtimes: Vec<SageAppRuntimeRecordView>,
}

impl RuntimeManagerRuntimesChangedEvent {
    pub fn new(runtimes: Vec<SageAppRuntimeRecordView>) -> Self {
        Self { runtimes }
    }
}

impl AppRuntimeEvent for RuntimeManagerRuntimesChangedEvent {
    const TYPE: &'static str = "runtimeManager.runtimesChanged";
    const RAIL: AppRuntimeEventRail = AppRuntimeEventRail::System;
}
