use serde::Serialize;
use specta::Type;
use crate::system_apps::SystemAppUsage;
use crate::types::app::common::SageAppCommon;
use crate::types::permissions::SageGrantedSystemPermissions;
use crate::types::SageApp;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum AppPresentation {
    Taskbar,
    Modal(AppModalPresentation),
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppModalPresentation {
    visible_over_app_ids: Vec<String>,
    visible_over_launchpad: bool,
}

#[derive(Debug)]
pub struct SystemSageApp {
    common: SageAppCommon,
    usage: SystemAppUsage,
    system_granted_permissions: SageGrantedSystemPermissions,
}

impl SystemSageApp {
    pub fn new(
        common: SageAppCommon,
        usage: SystemAppUsage,
        system_granted_permissions: SageGrantedSystemPermissions,
    ) -> Self {
        Self {
            common,
            usage,
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

    pub fn usage(&self) -> SystemAppUsage {
        self.usage
    }

    pub fn system_granted_permissions(&self) -> &SageGrantedSystemPermissions {
        &self.system_granted_permissions
    }
}

impl AppModalPresentation {
    pub fn over_apps(app_ids: Vec<String>) -> Self {
        Self {
            visible_over_app_ids: app_ids,
            visible_over_launchpad: false,
        }
    }

    pub fn over_app_and_launchpad(app_id: String) -> Self {
        Self {
            visible_over_app_ids: vec![app_id],
            visible_over_launchpad: true,
        }
    }

    pub fn over_launchpad() -> Self {
        Self {
            visible_over_app_ids: vec![],
            visible_over_launchpad: true,
        }
    }

    pub fn visible_over_app_ids(&self) -> Vec<String> {
        self.visible_over_app_ids.clone()
    }

    pub fn visible_over_launchpad(&self) -> bool {
        self.visible_over_launchpad
    }

    pub fn update_app_ids(&mut self, target_app_ids: Vec<String>) {
        self.visible_over_app_ids = target_app_ids;
    }
}
