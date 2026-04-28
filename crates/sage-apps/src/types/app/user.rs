use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

use crate::sandbox::SANDBOX_TEST_ID_PREFIX;
use crate::types::SageAppUrl;
use crate::types::app::common::SageAppCommon;
use crate::types::app::flags::SageAppFlags;
use crate::types::app::preview::UserSageAppPendingUpdate;
use crate::types::app::snapshot::SageAppSnapshot;
use crate::types::app::system::SystemSageApp;
use crate::types::permissions::{
    SageGrantedPermissions, SageGrantedSystemPermissions, SageRequestedPermissions,
};
use crate::types::storage::InstalledSageAppStorage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UserSageAppSource {
    Zip,
    Url { app_url: SageAppUrl },
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageApp {
    common: SageAppCommon,
    source: UserSageAppSource,
    pending_update: Option<UserSageAppPendingUpdate>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum SageApp {
    System(SystemSageApp),
    User(UserSageApp),
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CorruptedInstalledSageApp {
    id: String,
    app_dir: String,
    error: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ListedSageApp {
    User(UserSageApp),
    System(SystemSageApp),
    Corrupted(CorruptedInstalledSageApp),
}

impl UserSageAppSource {
    pub fn url(app_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let app_url = SageAppUrl::parse(app_url.as_ref())?;

        Ok(Self::Url { app_url })
    }
}

impl UserSageApp {
    pub fn new_installed(common: SageAppCommon, source: UserSageAppSource) -> Self {
        Self {
            common,
            source,
            pending_update: None,
        }
    }

    pub fn set_pending_update(&mut self, pending_update: Option<UserSageAppPendingUpdate>) {
        self.pending_update = pending_update;
    }

    pub fn into_sage_app(self) -> SageApp {
        SageApp::User(self)
    }

    pub fn common(&self) -> &SageAppCommon {
        &self.common
    }

    pub fn common_mut(&mut self) -> &mut SageAppCommon {
        &mut self.common
    }

    pub fn source(&self) -> &UserSageAppSource {
        &self.source
    }

    pub fn pending_update(&self) -> Option<&UserSageAppPendingUpdate> {
        self.pending_update.as_ref()
    }

    pub fn app_path(&self) -> PathBuf {
        self.common().app_path()
    }
}

impl SageApp {
    pub fn common(&self) -> &SageAppCommon {
        match self {
            Self::System(app) => app.common(),
            Self::User(app) => app.common(),
        }
    }

    pub fn common_mut(&mut self) -> &mut SageAppCommon {
        match self {
            Self::System(app) => app.common_mut(),
            Self::User(app) => app.common_mut(),
        }
    }

    pub fn id(&self) -> &str {
        self.common().id()
    }

    pub fn origin_id(&self) -> &str {
        self.common().origin_id()
    }

    pub fn name(&self) -> &str {
        self.common().name()
    }

    pub fn version(&self) -> &str {
        self.common().version()
    }

    pub fn app_dir(&self) -> &str {
        self.common().app_dir()
    }

    pub fn app_path(&self) -> PathBuf {
        self.common().app_path()
    }

    pub fn entry_file(&self) -> &str {
        self.common().entry_file()
    }

    pub fn icon_file(&self) -> Option<&str> {
        self.common().icon_file()
    }

    pub fn requested_permissions(&self) -> &SageRequestedPermissions {
        self.common().requested_permissions()
    }

    pub fn granted_permissions(&self) -> &SageGrantedPermissions {
        self.common().granted_permissions()
    }

    pub fn system_granted_permissions(&self) -> Option<&SageGrantedSystemPermissions> {
        match self {
            Self::System(app) => Some(app.system_granted_permissions()),
            Self::User(_) => None,
        }
    }

    pub fn capability_flags(&self) -> &SageAppFlags {
        self.common().capability_flags()
    }

    pub fn storage(&self) -> &InstalledSageAppStorage {
        self.common().storage()
    }

    pub fn active_snapshot(&self) -> &SageAppSnapshot {
        self.common().active_snapshot()
    }

    pub fn as_user(&self) -> Option<&UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn as_system(&self) -> Option<&SystemSageApp> {
        match self {
            Self::System(app) => Some(app),
            Self::User(_) => None,
        }
    }

    pub fn into_user(self) -> Option<UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn into_system(self) -> Option<SystemSageApp> {
        match self {
            Self::System(app) => Some(app),
            Self::User(_) => None,
        }
    }

    pub fn is_sandbox_test(&self) -> bool {
        self.id().starts_with(SANDBOX_TEST_ID_PREFIX)
    }
}

impl CorruptedInstalledSageApp {
    pub(crate) fn new(
        id: impl Into<String>,
        app_dir: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            app_dir: app_dir.into(),
            error: error.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn app_dir(&self) -> &str {
        &self.app_dir
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}
