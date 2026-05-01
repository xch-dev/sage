use serde::{Serialize};
use specta::Type;
use crate::types::{SystemAppPresentation, SystemSageApp};
use crate::types::app::view::common::SageAppCommonView;
use crate::types::app::view::permission::SageGrantedSystemPermissionsView;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemSageAppView {
    common: SageAppCommonView,
    presentation: SystemAppPresentation,
    system_granted_permissions: SageGrantedSystemPermissionsView,
}

impl From<&SystemSageApp> for SystemSageAppView {
    fn from(value: &SystemSageApp) -> Self {
        Self {
            common: value.common().into(),
            presentation: value.presentation(),
            system_granted_permissions: value.system_granted_permissions().into(),
        }
    }
}
