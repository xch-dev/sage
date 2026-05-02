use crate::capabilities::list::UserBridgeCapability;
use crate::runtime::SharedRuntime;
use crate::types::app::common::SageAppCommon;
use crate::types::app::flags::SageAppFlags;
use crate::types::app::preview::UserSageAppPendingUpdate;
use crate::types::app::snapshot::SageAppSnapshot;
use crate::types::app::system_apps::SystemSageApp;
use crate::types::permissions::{
    SageGrantedPermissions, SageGrantedSystemPermissions, SageRequestedPermissions,
};
use crate::types::storage::InstalledSageAppStorage;
use crate::types::SageAppUrl;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;

#[derive(Debug)]
pub struct ResolvedStoppedApp {
    app: SharedSageApp,
    _guard: OwnedMutexGuard<()>,
}

#[derive(Debug)]
pub struct ResolvedRunningApp {
    runtime: SharedRuntime,
}

impl ResolvedStoppedApp {
    pub fn new(app: SharedSageApp, _guard: OwnedMutexGuard<()>) -> Self {
        Self { app, _guard }
    }

    pub fn with_app<T>(&self, f: impl FnOnce(&SharedSageApp) -> T) -> T {
        f(&self.app)
    }

    pub fn try_with_app<T, E>(
        &self,
        f: impl FnOnce(&SharedSageApp) -> Result<T, E>,
    ) -> Result<T, E> {
        f(&self.app)
    }

    pub fn into_app(self) -> SharedSageApp {
        self.app
    }
}

impl ResolvedRunningApp {
    pub fn new(runtime: SharedRuntime) -> Self {
        Self { runtime }
    }

    pub fn runtime(&self) -> SharedRuntime {
        self.runtime.clone()
    }

    pub fn with_app<T>(&self, f: impl FnOnce(&SharedSageApp) -> T) -> T {
        let app = self.runtime.app();
        f(&app)
    }
}

#[derive(Debug)]
pub enum ResolvedApp {
    Stopped(ResolvedStoppedApp),
    Running(ResolvedRunningApp),
}

impl ResolvedApp {
    pub fn with_app<T>(&self, f: impl FnOnce(&SharedSageApp) -> T) -> T {
        match self {
            Self::Stopped(stopped) => stopped.with_app(f),
            Self::Running(running) => running.with_app(f),
        }
    }

    pub fn clone_app_for_operation(&self) -> SharedSageApp {
        match self {
            Self::Stopped(stopped) => stopped.app.clone_for_resolved_running_app(),
            Self::Running(running) => running.runtime.app(),
        }
    }
}

#[derive(Debug)]
pub struct SharedSageApp {
    inner: Arc<parking_lot::RwLock<SageApp>>,
}

impl SharedSageApp {
    pub fn new(app: SageApp) -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(app)),
        }
    }

    pub(crate) fn clone_for_runtime_owner(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub(crate) fn clone_for_resolved_running_app(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub fn with<T>(&self, f: impl FnOnce(&SageApp) -> T) -> T {
        let app = self.inner.read();
        f(&app)
    }

    pub fn with_mut<T>(&self, f: impl FnOnce(&mut SageApp) -> T) -> T {
        let mut app = self.inner.write();
        f(&mut app)
    }

    pub fn try_with<T, E>(&self, f: impl FnOnce(&SageApp) -> Result<T, E>) -> Result<T, E> {
        let app = self.inner.read();
        f(&app)
    }

    pub fn try_with_mut<T, E>(&self, f: impl FnOnce(&mut SageApp) -> Result<T, E>) -> Result<T, E> {
        let mut app = self.inner.write();
        f(&mut app)
    }

    pub fn is_user_app(&self) -> bool {
        self.with(|app| app.is_user())
    }

    pub fn is_system_app(&self) -> bool {
        self.with(|app| app.is_system())
    }

    pub fn id(&self) -> String {
        self.with(|app| app.id().to_string())
    }

    pub fn name(&self) -> String {
        self.with(|app| app.name().to_string())
    }

    pub fn origin_id(&self) -> String {
        self.with(|app| app.origin_id().to_string())
    }

    pub fn app_path(&self) -> PathBuf {
        self.with(|app| app.app_path())
    }

    pub fn source(&self) -> Option<UserSageAppSource> {
        self.with(|app| app.as_user().map(|user| user.source().clone()))
    }

    pub fn pending_update(&self) -> Option<UserSageAppPendingUpdate> {
        self.with(|app| app.as_user().and_then(|user| user.pending_update().cloned()))
    }

    pub fn is_capability_granted(&self, capability: UserBridgeCapability) -> bool {
        self.with(|app| app.granted_permissions().has_capability(capability))
    }

    pub fn has_secret_access(&self) -> bool {
        self.with(|app| app.flags().has_secret_access())
    }

    pub fn storage_may_contain_secrets(&self) -> bool {
        self.with(|app| app.flags().storage_may_contain_secrets())
    }

    pub fn webview_label_matches(&self, label: &str) -> bool {
        let app_id = self.id();

        if let Some(extracted_app_id) = label.strip_prefix("app-") {
            return self.is_user_app() && extracted_app_id == app_id;
        }

        if let Some(extracted_app_id) = label.strip_prefix("system-app-") {
            return self.is_system_app() && extracted_app_id == app_id;
        }

        false
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SageApp {
    System(SystemSageApp),
    User(UserSageApp),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UserSageAppSource {
    Zip,
    Url { app_url: SageAppUrl },
}

#[derive(Debug)]
pub struct UserSageApp {
    common: SageAppCommon,
    source: UserSageAppSource,
    pending_update: Option<UserSageAppPendingUpdate>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CorruptedInstalledSageApp {
    id: String,
    app_dir: String,
    error: String,
}

#[derive(Debug)]
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
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User(_))
    }

    pub fn is_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

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

    pub fn entry_file(&self) -> String {
        self.common().entry_file().to_string()
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

    pub fn flags(&self) -> &SageAppFlags {
        self.common().flags()
    }

    pub fn storage(&self) -> &InstalledSageAppStorage {
        self.common().storage()
    }

    pub fn active_snapshot(&self) -> &SageAppSnapshot {
        self.common().active_snapshot()
    }

    pub fn set_pending_update(
        &mut self,
        pending_update: Option<UserSageAppPendingUpdate>,
    ) -> anyhow::Result<()> {
        match self {
            Self::User(app) => {
                app.set_pending_update(pending_update);
                Ok(())
            }
            Self::System(_) => anyhow::bail!("system app cannot have pending user update"),
        }
    }

    pub fn as_user(&self) -> Option<&UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn as_user_mut(&mut self) -> Option<&mut UserSageApp> {
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
