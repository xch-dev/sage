use serde::Serialize;
use specta::Type;

use crate::bridge::EnvironmentThemeView;
use crate::bridge::UserRuntimeEvent;
use crate::capabilities::UserBridgeCapability;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentThemeChangedEvent {
    pub theme: EnvironmentThemeView,
}

impl UserRuntimeEvent for EnvironmentThemeChangedEvent {
    const TYPE: &'static str = "environment.theme.changed";
    const REQUIRED_CAPABILITY: UserBridgeCapability =
        UserBridgeCapability::EnvironmentThemeListenChanged;
}
