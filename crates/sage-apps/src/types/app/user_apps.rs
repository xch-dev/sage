use crate::capabilities::list::BridgeCapability;
use crate::lifecycle::write_metadata_for_app;
use crate::runtime::SharedRuntime;
use crate::types::SageAppUrl;
use crate::types::app::common::SageAppCommon;
use crate::types::app::flags::SageAppFlags;
use crate::types::app::preview::UserSageAppPendingUpdate;
use crate::types::app::snapshot::SageAppSnapshot;
use crate::types::app::system_apps::SystemSageApp;
use crate::types::app::view::SageAppIconView;
use crate::types::permissions::{
    SageGrantedPermissions, SageGrantedSystemPermissions, SageRequestedPermissions,
};
use crate::types::storage::InstalledSageAppStorage;
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

    pub fn try_with<T, E>(&self, f: impl FnOnce(&SageApp) -> Result<T, E>) -> Result<T, E> {
        let app = self.inner.read();
        f(&app)
    }
    pub fn try_mutate<T, E>(
        &self,
        f: impl FnOnce(&mut SageApp) -> Result<T, E>,
    ) -> Result<T, String>
    where
        E: ToString,
    {
        let mut app = self.inner.write();

        let previous_app = app.clone_for_rollback().map_err(|err| err.to_string())?;

        match f(&mut app) {
            Ok(value) => {
                if let Err(err) = write_metadata_for_app(&app) {
                    *app = previous_app;
                    return Err(format!("failed to persist app metadata: {err}"));
                }

                Ok(value)
            }

            Err(err) => {
                *app = previous_app;
                Err(err.to_string())
            }
        }
    }

    pub fn is_user_app(&self) -> bool {
        self.with(SageApp::is_user)
    }

    pub fn is_system_app(&self) -> bool {
        self.with(SageApp::is_system)
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
        self.with(SageApp::app_path)
    }

    pub fn source(&self) -> Option<UserSageAppSource> {
        self.with(|app| app.as_user().map(|user| user.source().clone()))
    }

    pub fn pending_update(&self) -> Option<UserSageAppPendingUpdate> {
        self.with(|app| {
            app.as_user()
                .and_then(|user| user.pending_update().cloned())
        })
    }

    pub fn is_capability_granted(&self, capability: BridgeCapability) -> bool {
        self.with(|app| match capability {
            BridgeCapability::User(capability) => {
                app.granted_permissions().has_capability(capability)
            }

            BridgeCapability::System(capability) => app
                .system_granted_permissions()
                .is_some_and(|permissions| permissions.capabilities().contains(&capability)),
        })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<SageAppIconView>,

    app_dir: String,
    error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    manifest_header: Option<crate::types::SageAppManifestHeaderV0>,

    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<UserSageAppSource>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ListedSageApp {
    User(UserSageApp),
    System(SystemSageApp),
    Corrupted(CorruptedInstalledSageApp),
}

impl UserSageApp {
    pub(crate) fn new_installed(common: SageAppCommon, source: UserSageAppSource) -> Self {
        Self {
            common,
            source,
            pending_update: None,
        }
    }

    pub(crate) fn clone_for_rollback(&self) -> Self {
        Self {
            common: self.common.clone_for_rollback(),
            source: self.source.clone(),
            pending_update: self.pending_update.clone(),
        }
    }

    pub(crate) fn set_pending_update(&mut self, pending_update: Option<UserSageAppPendingUpdate>) {
        self.pending_update = pending_update;
    }

    pub(crate) fn common(&self) -> &SageAppCommon {
        &self.common
    }

    pub(crate) fn common_mut(&mut self) -> &mut SageAppCommon {
        &mut self.common
    }

    pub(crate) fn source(&self) -> &UserSageAppSource {
        &self.source
    }

    pub(crate) fn pending_update(&self) -> Option<&UserSageAppPendingUpdate> {
        self.pending_update.as_ref()
    }
}

impl SageApp {
    pub(crate) fn is_user(&self) -> bool {
        matches!(self, Self::User(_))
    }

    pub(crate) fn is_system(&self) -> bool {
        matches!(self, Self::System(_))
    }

    pub(crate) fn common(&self) -> &SageAppCommon {
        match self {
            Self::System(app) => app.common(),
            Self::User(app) => app.common(),
        }
    }

    pub(crate) fn common_mut(&mut self) -> &mut SageAppCommon {
        match self {
            Self::System(app) => app.common_mut(),
            Self::User(app) => app.common_mut(),
        }
    }

    pub(crate) fn clone_for_rollback(&self) -> anyhow::Result<Self> {
        match self {
            Self::User(app) => Ok(Self::User(app.clone_for_rollback())),
            Self::System(_) => {
                anyhow::bail!("system apps are immutable")
            }
        }
    }

    pub(crate) fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        let Self::User(app) = self else {
            anyhow::bail!("system app cannot receive user update");
        };

        app.common_mut()
            .apply_update(pending, granted_permissions, snapshot)
            .map_err(|err| anyhow::anyhow!("failed to apply app update: {err}"))?;

        app.set_pending_update(None);

        Ok(())
    }

    pub(crate) fn id(&self) -> &str {
        self.common().id()
    }

    pub(crate) fn origin_id(&self) -> &str {
        self.common().origin_id()
    }

    pub(crate) fn name(&self) -> &str {
        self.common().name()
    }

    pub(crate) fn version(&self) -> &str {
        self.common().version()
    }

    pub(crate) fn app_path(&self) -> PathBuf {
        self.common().app_path()
    }

    pub(crate) fn entry_file(&self) -> String {
        self.common().entry_file().to_string()
    }

    pub(crate) fn requested_permissions(&self) -> &SageRequestedPermissions {
        self.common().requested_permissions()
    }

    pub(crate) fn granted_permissions(&self) -> &SageGrantedPermissions {
        self.common().granted_permissions()
    }

    pub(crate) fn system_granted_permissions(&self) -> Option<&SageGrantedSystemPermissions> {
        match self {
            Self::System(app) => Some(app.system_granted_permissions()),
            Self::User(_) => None,
        }
    }

    pub(crate) fn flags(&self) -> &SageAppFlags {
        self.common().flags()
    }

    pub(crate) fn storage(&self) -> &InstalledSageAppStorage {
        self.common().storage()
    }

    pub(crate) fn active_snapshot(&self) -> &SageAppSnapshot {
        self.common().active_snapshot()
    }

    pub(crate) fn set_pending_update(
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

    pub(crate) fn as_user(&self) -> Option<&UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
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
            icon: None,
            app_dir: app_dir.into(),
            error: error.into(),
            manifest_header: None,
            source: None,
        }
    }

    pub(crate) fn with_icon(mut self, icon: Option<SageAppIconView>) -> Self {
        self.icon = icon;
        self
    }

    pub(crate) fn with_manifest_header(
        mut self,
        manifest_header: Option<crate::types::SageAppManifestHeaderV0>,
    ) -> Self {
        self.manifest_header = manifest_header;
        self
    }

    pub(crate) fn with_source(mut self, source: Option<UserSageAppSource>) -> Self {
        self.source = source;
        self
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }
}

impl UserSageApp {
    pub(crate) fn into_sage_app(self) -> SageApp {
        SageApp::User(self)
    }
}

#[cfg(test)]
impl UserSageAppSource {
    pub(crate) fn url(app_url: impl AsRef<str>) -> anyhow::Result<Self> {
        let app_url = SageAppUrl::parse(app_url.as_ref())?;
        Ok(Self::Url { app_url })
    }
}

#[cfg(test)]
impl CorruptedInstalledSageApp {
    pub(crate) fn error(&self) -> &str {
        &self.error
    }
}
