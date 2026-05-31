use serde::Serialize;
use specta::Type;

use crate::types::SageAppCommonView;
use crate::types::SageGrantedSystemPermissionsView;
use crate::types::SystemSageApp;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemSageAppView {
    common: SageAppCommonView,
    system_granted_permissions: SageGrantedSystemPermissionsView,
}

impl From<&SystemSageApp> for SystemSageAppView {
    fn from(value: &SystemSageApp) -> Self {
        Self {
            common: value.common().into(),
            system_granted_permissions: value.system_granted_permissions().into(),
        }
    }
}
