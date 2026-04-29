use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::SageApp;
use crate::types::app::common::SageAppCommon;
use crate::types::permissions::SageGrantedSystemPermissions;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum SystemAppPresentation {
    Taskbar,
    Modal,
}

#[derive(Debug)]
pub struct SystemSageApp {
    common: SageAppCommon,
    system_granted_permissions: SageGrantedSystemPermissions,
    presentation: SystemAppPresentation,
}

impl SystemSageApp {
    pub fn new(
        common: SageAppCommon,
        system_granted_permissions: SageGrantedSystemPermissions,
        presentation: SystemAppPresentation,
    ) -> Self {
        Self {
            common,
            system_granted_permissions,
            presentation,
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

    pub fn presentation(&self) -> SystemAppPresentation {
        self.presentation
    }
}
