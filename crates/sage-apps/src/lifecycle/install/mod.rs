use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use tauri::AppHandle;

use crate::lifecycle::{
    allocate_new_storage, apps_root, manifest_entry_file, manifest_icon_file,
    write_installed_app_metadata,
};
use crate::lifecycle::flags::get_app_flags;
use crate::types::{
    InstalledSageAppStorage, SageAppFlags, SageAppCommon, SageAppPackageManifest,
    SageAppSnapshot, SageGrantedPermissions, UserSageApp, UserSageAppSource,
};

pub mod commands;
pub mod url;
pub mod zip;

#[async_trait]
pub trait AppInstallSource {
    type Prepared: Send + Sync;

    async fn prepare(&self) -> AnyResult<Self::Prepared>;

    fn manifest<'a>(&self, prepared: &'a Self::Prepared) -> &'a SageAppPackageManifest;

    fn source(&self, prepared: &Self::Prepared) -> UserSageAppSource;

    fn resolve_target(
        &self,
        root: &Path,
        base_path: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)>;

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<SageAppSnapshot>;

    fn origin_id(
        &self,
        _base_path: &Path,
        app_id: &str,
        existing: Option<&UserSageApp>,
    ) -> AnyResult<String> {
        Ok(existing
            .map(|app| app.common.origin_id.clone())
            .unwrap_or_else(|| app_id.to_string()))
    }

    fn after_origin_selected(
        &self,
        _base_path: &Path,
        _app_id: &str,
        _origin_id: &str,
    ) -> AnyResult<()> {
        Ok(())
    }
}

trait InstallStorageResolver {
    fn resolve_storage(
        &self,
        existing: Option<&UserSageApp>,
    ) -> AnyResult<Option<InstalledSageAppStorage>>;
}

struct TauriStorageResolver;

impl InstallStorageResolver for TauriStorageResolver {
    fn resolve_storage(
        &self,
        existing: Option<&UserSageApp>,
    ) -> AnyResult<Option<InstalledSageAppStorage>> {
        Ok(existing.map(|app| app.common.storage.clone()))
    }
}

pub async fn install_app_from_source<S>(
    app: &AppHandle,
    base_path: &Path,
    granted_permissions: SageGrantedPermissions,
    source: S,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    install_app_from_source_with_storage(
        base_path,
        granted_permissions,
        source,
        &TauriStorageResolver,
        Some(app),
    )
        .await
}

async fn install_app_from_source_with_storage<S, R>(
    base_path: &Path,
    granted_permissions: SageGrantedPermissions,
    source: S,
    storage_resolver: &R,
    app: Option<&AppHandle>,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
    R: InstallStorageResolver + Sync,
{
    let root = apps_root(base_path);
    fs::create_dir_all(&root)?;

    let prepared = source.prepare().await?;
    let manifest = source.manifest(&prepared);

    let granted_permissions = SageGrantedPermissions::from_requested_and_granted(
        &manifest.permissions(),
        granted_permissions,
    )?;

    let effective_capabilities = manifest
        .permissions()
        .capabilities
        .resolve_effective_grants(granted_permissions.capabilities().copied())?;

    let app_flags = get_app_flags(&effective_capabilities, None)?;

    let (app_id, app_dir, existing_app) =
        source.resolve_target(&root, base_path, &prepared)?;

    let storage = match storage_resolver.resolve_storage(existing_app.as_ref())? {
        Some(storage) => storage,
        None => {
            let app = app.expect("missing AppHandle for new install storage allocation");
            allocate_new_storage(app, base_path).await?
        }
    };

    recreate_app_dir(&app_dir)?;

    let snapshot = source.create_snapshot(&app_dir, &prepared).await?;

    let origin_id = source.origin_id(base_path, &app_id, existing_app.as_ref())?;

    source.after_origin_selected(base_path, &app_id, &origin_id)?;

    let installed = build_installed_app(
        app_id.clone(),
        origin_id,
        &app_dir,
        manifest,
        granted_permissions,
        app_flags,
        storage,
        source.source(&prepared),
        snapshot,
    );

    write_installed_app_metadata(&installed, &app_dir)?;

    Ok(installed)
}

pub fn build_installed_app(
    app_id: String,
    origin_id: String,
    app_dir: &Path,
    manifest: &SageAppPackageManifest,
    granted_permissions: SageGrantedPermissions,
    permission_flags: SageAppFlags,
    storage: InstalledSageAppStorage,
    source: UserSageAppSource,
    snapshot: SageAppSnapshot,
) -> UserSageApp {
    UserSageApp {
        common: SageAppCommon {
            id: app_id,
            origin_id,
            name: manifest.name().to_string(),
            version: manifest.version().to_string(),
            app_dir: app_dir.to_string_lossy().to_string(),
            entry_file: manifest_entry_file(manifest).to_string(),
            icon_file: manifest_icon_file(manifest).to_string(),
            requested_permissions: manifest.permissions().clone(),
            granted_permissions,
            capability_flags: permission_flags,
            storage,
            active_snapshot: snapshot,
        },
        source,
        pending_update: None,
    }
}

pub fn recreate_app_dir(app_dir: &Path) -> AnyResult<()> {
    if app_dir.exists() {
        fs::remove_dir_all(app_dir)?;
    }

    fs::create_dir_all(app_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::registry::read_installed_app_by_id;
    use crate::types::{SageAppManifestFile, SageAppPackageManifestParts, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions};
    use tempfile::tempdir;

    struct TestStorageResolver {
        storage: InstalledSageAppStorage,
    }

    impl InstallStorageResolver for TestStorageResolver {
        fn resolve_storage(
            &self,
            _existing: Option<&UserSageApp>,
        ) -> AnyResult<Option<InstalledSageAppStorage>> {
            Ok(Some(self.storage.clone()))
        }
    }

    fn sample_manifest() -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".into(),
            version: "1.0.0".into(),
            permissions: SageRequestedPermissions::new(
                SageRequestedNetworkPermissions::new(
                    [SageNetworkWhitelistEntry::new_unchecked("https", "api.example.com")],
                    []
                ),
                SageRequestedCapabilities::new(
                    [UserBridgeCapability::PersistentStorage],
                    [UserBridgeCapability::WalletSendXch],
                )
            ).unwrap(),
            files: vec![SageAppManifestFile {
                path: "index.html".into(),
                sha256: "a".repeat(64),
                size: 123,
            }],
            entry: Some("index.html".into()),
            icon: Some("icon.png".into()),
            author: None,
            donation: None,
        }).unwrap()
    }

    struct FakeInstallSource {
        manifest: SageAppPackageManifest,
        app_id: String,
        origin_id: String,
        source: UserSageAppSource,
    }

    struct FakePrepared {
        manifest: SageAppPackageManifest,
    }

    #[async_trait]
    impl AppInstallSource for FakeInstallSource {
        type Prepared = FakePrepared;

        async fn prepare(&self) -> AnyResult<Self::Prepared> {
            Ok(FakePrepared {
                manifest: self.manifest.clone(),
            })
        }

        fn manifest<'a>(&self, prepared: &'a Self::Prepared) -> &'a SageAppPackageManifest {
            &prepared.manifest
        }

        fn source(&self, _prepared: &Self::Prepared) -> UserSageAppSource {
            self.source.clone()
        }

        fn resolve_target(
            &self,
            root: &Path,
            _base_path: &Path,
            _prepared: &Self::Prepared,
        ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
            let app_dir = root.join(&self.app_id);
            Ok((self.app_id.clone(), app_dir, None))
        }

        async fn create_snapshot(
            &self,
            app_dir: &Path,
            prepared: &Self::Prepared,
        ) -> AnyResult<SageAppSnapshot> {
            Ok(SageAppSnapshot {
                manifest_hash: "fake-hash".into(),
                snapshot_dir: app_dir.to_string_lossy().to_string(),
                total_bytes: 123,
                manifest: prepared.manifest.clone(),
            })
        }

        fn origin_id(
            &self,
            _base_path: &Path,
            _app_id: &str,
            _existing: Option<&UserSageApp>,
        ) -> AnyResult<String> {
            Ok(self.origin_id.clone())
        }
    }

    #[tokio::test]
    async fn shared_installer_builds_and_writes_installed_app() {
        let dir = tempdir().unwrap();

        let manifest = sample_manifest();

        let granted = SageGrantedPermissions::new(
            &manifest.permissions(),
            [UserBridgeCapability::PersistentStorage],
            [SageNetworkWhitelistEntry::new_unchecked("https", "api.example.com")],
        )
            .unwrap();

        let installed = install_app_from_source_with_storage(
            dir.path(),
            granted,
            FakeInstallSource {
                manifest,
                app_id: "fake-app".into(),
                origin_id: "fake-origin".into(),
                source: UserSageAppSource::Zip,
            },
            &TestStorageResolver {
                storage: InstalledSageAppStorage::Unmanaged,
            },
            None,
        )
            .await
            .unwrap();

        assert_eq!(installed.common.id, "fake-app");
        assert_eq!(installed.common.origin_id, "fake-origin");
        assert_eq!(installed.common.name, "Test App");
        assert_eq!(installed.common.entry_file, "index.html");
        assert_eq!(installed.common.icon_file, "icon.png");
        assert_eq!(installed.common.storage, InstalledSageAppStorage::Unmanaged);
        assert_eq!(installed.source, UserSageAppSource::Zip);

        let reread = read_installed_app_by_id(dir.path(), "fake-app").unwrap();
        assert_eq!(reread.common.id, "fake-app");
        assert_eq!(reread.common.origin_id, "fake-origin");
    }

    #[tokio::test]
    async fn shared_installer_rejects_unrequested_granted_permission() {
        let dir = tempdir().unwrap();

        let granted = SageGrantedPermissions::new_unchecked(
            [UserBridgeCapability::PersistentStorage],
            [SageNetworkWhitelistEntry::new_unchecked("https", "evil.example.com")],
        );

        let err = install_app_from_source_with_storage(
            dir.path(),
            granted,
            FakeInstallSource {
                manifest: sample_manifest(),
                app_id: "fake-app".into(),
                origin_id: "fake-origin".into(),
                source: UserSageAppSource::Zip,
            },
            &TestStorageResolver {
                storage: InstalledSageAppStorage::Unmanaged,
            },
            None,
        )
            .await
            .unwrap_err();

        assert!(err
            .to_string()
            .contains("granted network whitelist entry not requested"));
    }

    #[test]
    fn build_installed_app_sets_id_and_origin_id_independently() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("url-abc123");
        fs::create_dir_all(&app_dir).unwrap();

        let manifest = sample_manifest();

        let granted_permissions = SageGrantedPermissions::new(
            &manifest.permissions(),
            [UserBridgeCapability::PersistentStorage],
            [],
        )
            .unwrap();

        let app = build_installed_app(
            "url-abc123".into(),
            "r123-url-abc123".into(),
            &app_dir,
            &manifest,
            granted_permissions,
            SageAppFlags::default(),
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::Zip,
            SageAppSnapshot {
                manifest_hash: "hash".into(),
                snapshot_dir: app_dir.to_string_lossy().to_string(),
                total_bytes: 1,
                manifest: manifest.clone(),
            },
        );

        assert_eq!(app.common.id, "url-abc123");
        assert_eq!(app.common.origin_id, "r123-url-abc123");
    }
}
