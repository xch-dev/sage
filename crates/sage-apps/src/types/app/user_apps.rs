use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::OwnedMutexGuard;

use crate::{
    BridgeCapability, SageAppCommon, SageAppCommonRaw, SageAppIconView, SageAppManifestHeaderV0,
    SageAppSnapshot, SageAppStorage, SageAppUrl, SageGrantedPermissions,
    SageGrantedSystemPermissions, SageRequestedPermissions, SharedRuntime, SystemSageApp,
    UserSageAppPendingUpdate, UserSageAppPendingUpdateView,
};

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

    pub fn into_app(self) -> SharedSageApp {
        self.app
    }

    /// Returns the app without releasing its per-app operation lock.
    pub fn into_app_and_guard(self) -> (SharedSageApp, OwnedMutexGuard<()>) {
        (self.app, self._guard)
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

    #[allow(clippy::wrong_self_convention)]
    pub fn into_app(&self) -> SharedSageApp {
        self.runtime.app().clone()
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
            Self::Stopped(stopped) => stopped.app.clone(),
            Self::Running(running) => running.runtime.app(),
        }
    }
}

#[derive(Debug)]
pub struct SharedSageApp {
    inner: Arc<parking_lot::RwLock<SageApp>>,
}

impl SharedSageApp {
    pub(crate) fn new(app: SageApp) -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(app)),
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub(crate) fn replace_committed(&self, next: SageApp) {
        let mut app = self.inner.write();
        *app = next;
    }

    pub(crate) fn should_review_pending_update(&self) -> bool {
        self.with(|sage_app| {
            let Some(user_app) = sage_app.as_user() else {
                return false;
            };

            user_app.pending_update().is_some_and(|pending| {
                UserSageAppPendingUpdateView::from_pending_update(
                    pending,
                    user_app.common().granted_permissions(),
                )
                .decision()
                .is_review()
            })
        })
    }

    pub(crate) fn runtime_can_persist_secrets(&self) -> bool {
        self.with(|app| {
            app.common().has_secret_access() && app.common().has_persistent_webview_storage()
        })
    }

    pub(crate) fn with<T>(&self, f: impl FnOnce(&SageApp) -> T) -> T {
        let app = self.inner.read();
        f(&app)
    }

    pub(crate) fn try_with<T, E>(&self, f: impl FnOnce(&SageApp) -> Result<T, E>) -> Result<T, E> {
        let app = self.inner.read();
        f(&app)
    }

    pub fn is_user_app(&self) -> bool {
        self.with(SageApp::is_user)
    }

    pub fn is_system_app(&self) -> bool {
        self.with(SageApp::is_system)
    }

    pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool {
        self.with(|app| app.common().is_wallet_in_scope(fingerprint))
    }

    pub(crate) fn id(&self) -> String {
        self.with(|app| app.id().to_string())
    }

    pub(crate) fn name(&self) -> String {
        self.with(|app| app.name().to_string())
    }

    pub(crate) fn origin_id(&self) -> String {
        self.with(|app| app.origin_id().to_string())
    }

    pub(crate) fn is_capability_granted(&self, capability: BridgeCapability) -> bool {
        self.with(|app| match capability {
            BridgeCapability::User(capability) => {
                app.granted_permissions().has_capability(capability)
            }

            BridgeCapability::System(capability) => app
                .system_granted_permissions()
                .is_some_and(|permissions| permissions.capabilities().contains(&capability)),
        })
    }

    pub(crate) fn webview_label(&self) -> String {
        self.with(|app| {
            if app.is_system() {
                format!("system-app-{}", app.id())
            } else {
                format!("app-{}", app.id())
            }
        })
    }

    pub(crate) fn webview_label_matches(&self, label: &str) -> bool {
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

#[derive(Debug, Serialize)]
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
    manifest_header: Option<SageAppManifestHeaderV0>,

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

    pub(crate) fn load_persisted(
        common: SageAppCommon,
        source: UserSageAppSource,
        pending_update: Option<UserSageAppPendingUpdate>,
    ) -> Self {
        Self {
            common,
            source,
            pending_update,
        }
    }

    pub(crate) fn clone_durable(&self) -> Self {
        Self {
            common: self.common.clone_durable(),
            source: self.source.clone(),
            pending_update: self.pending_update.clone(),
        }
    }

    pub(crate) fn set_pending_update(
        &mut self,
        pending_update: Option<UserSageAppPendingUpdate>,
    ) -> anyhow::Result<()> {
        if let Some(pending_update) = &pending_update {
            crate::validate_network_permissions_for_source(
                pending_update.manifest().permissions(),
                &self.source,
            )?;
        }

        self.pending_update = pending_update;
        Ok(())
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
            Self::User(app) => Ok(Self::User(app.clone_durable())),
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

        app.set_pending_update(None)?;

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

    pub(crate) fn storage(&self) -> &SageAppStorage {
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
            Self::User(app) => app.set_pending_update(pending_update),
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
        manifest_header: Option<SageAppManifestHeaderV0>,
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

#[derive(Debug, Deserialize)]
struct UserSageAppRaw {
    common: SageAppCommonRaw,
    source: UserSageAppSource,

    #[serde(default)]
    pending_update: Option<UserSageAppPendingUpdate>,
}

impl<'de> Deserialize<'de> for UserSageApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = UserSageAppRaw::deserialize(deserializer)?;

        let common: SageAppCommon = raw.common.try_into().map_err(serde::de::Error::custom)?;

        crate::validate_network_permissions_for_source(common.requested_permissions(), &raw.source)
            .map_err(serde::de::Error::custom)?;

        if let Some(pending_update) = &raw.pending_update {
            crate::validate_network_permissions_for_source(
                pending_update.manifest().permissions(),
                &raw.source,
            )
            .map_err(serde::de::Error::custom)?;
        }

        Ok(UserSageApp::load_persisted(
            common,
            raw.source,
            raw.pending_update,
        ))
    }
}

impl UserSageAppSource {
    pub(crate) fn allows_http_network_permissions(&self) -> bool {
        matches!(self, Self::Url { app_url } if app_url.is_loopback())
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
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::sync::Arc;

    use tempfile::{TempDir, tempdir};
    use tokio::sync::Mutex;

    use super::*;
    use crate::{
        SageAppIdentity, SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion,
        SageAppPackageManifest, SageAppPackageManifestParts, SageAppWalletScope,
    };

    fn test_app() -> (SharedSageApp, TempDir) {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "x").unwrap();

        let requested_permissions = SageRequestedPermissions::default();
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(0),
            name: "test app".to_string(),
            icon: None,
            sage_version: SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
            version: "1.0.0".to_string(),
            permissions: requested_permissions.clone(),
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 1).unwrap()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap();
        let granted_permissions =
            SageGrantedPermissions::new(&requested_permissions, [], [], BTreeMap::new()).unwrap();
        let snapshot =
            SageAppSnapshot::new("hash", dir.path().to_string_lossy(), manifest).unwrap();
        let common = SageAppCommon::new(
            SageAppIdentity::new("app-id", "origin-id", dir.path().to_string_lossy()).unwrap(),
            granted_permissions,
            SageAppStorage::Unmanaged,
            snapshot,
            SageAppWalletScope::AllWallets,
        )
        .unwrap();
        let app = UserSageApp::new_installed(common, UserSageAppSource::Zip);

        (SharedSageApp::new(app.into_sage_app()), dir)
    }

    #[tokio::test]
    async fn stopped_app_guard_can_be_held_after_extracting_app() {
        let lock = Arc::new(Mutex::new(()));
        let guard = lock.clone().lock_owned().await;
        let (app, _dir) = test_app();
        let resolved = ResolvedStoppedApp::new(app, guard);

        let (_app, guard) = resolved.into_app_and_guard();

        assert!(lock.clone().try_lock_owned().is_err());
        drop(guard);
        assert!(lock.try_lock_owned().is_ok());
    }
}
