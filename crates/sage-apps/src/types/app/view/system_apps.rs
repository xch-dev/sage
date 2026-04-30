use serde::{Deserialize, Serialize};
use specta::Type;
use crate::types::{SageGrantedSystemPermissions, SystemAppPresentation, SystemSageApp};
use crate::types::app::view::common::SageAppCommonView;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemSageAppView {
    common: SageAppCommonView,
    presentation: SystemAppPresentation,
    system_granted_permissions: SageGrantedSystemPermissions,
}

impl From<&SystemSageApp> for SystemSageAppView {
    fn from(value: &SystemSageApp) -> Self {
        Self {
            common: value.common().into(),
            presentation: value.presentation(),
            system_granted_permissions: value.system_granted_permissions().clone(),
        }
    }
}
