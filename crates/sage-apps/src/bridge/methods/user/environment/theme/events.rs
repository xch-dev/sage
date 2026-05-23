use crate::bridge::event_emit::UserRuntimeEvent;
use crate::bridge::methods::user::environment::EnvironmentThemeView;
use crate::capabilities::list::UserBridgeCapability;
use serde::Serialize;
use specta::Type;

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
