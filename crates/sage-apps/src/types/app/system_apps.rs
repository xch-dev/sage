use serde::Serialize;
use specta::Type;

use crate::types::app::common::SageAppCommon;
use crate::types::permissions::SageGrantedSystemPermissions;
use crate::types::SageApp;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub enum AppPresentation {
    Taskbar,
    Modal(AppModalPresentation),
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppModalPresentation {
    pub visible_over_app_ids: Vec<String>,
    pub visible_over_launchpad: bool,
}

#[derive(Debug)]
pub struct SystemSageApp {
    common: SageAppCommon,
    system_granted_permissions: SageGrantedSystemPermissions,
}

impl SystemSageApp {
    pub fn new(
        common: SageAppCommon,
        system_granted_permissions: SageGrantedSystemPermissions,
    ) -> Self {
        Self {
            common,
            system_granted_permissions,
        }
    }

    pub fn into_sage_app(self) -> SageApp {
        SageApp::System(self)
    }

    pub fn common(&self) -> &SageAppCommon {
        &self.common
    }

    pub fn common_mut(&mut self) -> &mut SageAppCommon {
        &mut self.common
    }

    pub fn system_granted_permissions(&self) -> &SageGrantedSystemPermissions {
        &self.system_granted_permissions
    }
}

impl AppModalPresentation {
    pub fn over_app(app_id: String) -> Self {
        Self {
            visible_over_app_ids: vec![app_id],
            visible_over_launchpad: false,
        }
    }

    pub fn over_app_and_launchpad(app_id: String) -> Self {
        Self {
            visible_over_app_ids: vec![app_id],
            visible_over_launchpad: true,
        }
    }
}
