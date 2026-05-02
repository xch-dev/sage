use serde::{Deserialize, Serialize};
use specta::Type;
use crate::bridge::event_emit::{AppRuntimeEvent, AppRuntimeEventRail};
use crate::bridge::methods::user::environment::EnvironmentThemeView;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentThemeChangedEvent {
    pub theme: EnvironmentThemeView,
}

impl AppRuntimeEvent for EnvironmentThemeChangedEvent {
    const TYPE: &'static str = "environment.theme.changed";
    const RAIL: AppRuntimeEventRail = AppRuntimeEventRail::User;
}
