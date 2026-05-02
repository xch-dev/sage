use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use tauri::AppHandle;

use crate::lifecycle::{allocate_new_storage, apps_root, write_user_app_metadata};
use crate::types::{InstalledSageAppStorage, SageAppCommon, SageAppIdentity, SageAppPackageManifest, SageAppSnapshot, SageGrantedPermissionsInput, UserSageApp, UserSageAppSource};

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
        if let Some(app) = existing {
            return Ok(app.common().origin_id().to_string());
        }

        Ok(app_id.to_string())
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
        Ok(existing.map(|app| app.common().storage().clone()))
    }
}

pub async fn install_app_from_source<S>(
    app: &AppHandle,
    base_path: &Path,
    granted_permissions_input: SageGrantedPermissionsInput,
    source: S,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    install_app_from_source_with_storage(
        base_path,
        granted_permissions_input,
        source,
        &TauriStorageResolver,
        Some(app),
    )
    .await
}

async fn install_app_from_source_with_storage<S, R>(
    base_path: &Path,
    sage_granted_permissions_input: SageGrantedPermissionsInput,
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

    let granted_permissions = sage_granted_permissions_input.resolve(manifest.permissions())?;

    let (app_id, app_dir, existing_app) = source.resolve_target(&root, base_path, &prepared)?;

    let storage = if let Some(storage) = storage_resolver.resolve_storage(existing_app.as_ref())? {
        storage
    } else {
        let app = app.expect("missing AppHandle for new install storage allocation");
        allocate_new_storage(app, base_path).await?
    };

    recreate_app_dir(&app_dir)?;

    let snapshot = source.create_snapshot(&app_dir, &prepared).await?;

    let origin_id = source.origin_id(base_path, &app_id, existing_app.as_ref())?;

    source.after_origin_selected(base_path, &app_id, &origin_id)?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            app_id.clone(),
            origin_id,
            app_dir.to_string_lossy().to_string(),
        )?,
        granted_permissions,
        storage,
        snapshot,
    )?;

    let installed = UserSageApp::new_installed(common, source.source(&prepared));

    write_user_app_metadata(&installed)?;

    Ok(installed)
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
    use crate::capabilities::list::UserBridgeCapability;
    use crate::lifecycle::registry::read_installed_app_by_id;
    use crate::types::{SageAppCommon, SageAppIdentity, SageAppManifestFile, SageAppPackageManifestParts, SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions};
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
                    [SageNetworkWhitelistEntry::new_unchecked(
                        "https",
                        "api.example.com",
                    )],
                    [],
                ),
                SageRequestedCapabilities::new(
                    [UserBridgeCapability::StoragePersistentWebview],
                    [UserBridgeCapability::WalletSendXch],
                ),
            )
            .unwrap(),
            files: vec![
                SageAppManifestFile::new("index.html".to_string(), "a".repeat(64), 123).unwrap(),
            ],
            entry: Some("index.html".into()),
            icon: None,
            author: None,
            donation: None,
        })
        .unwrap()
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
            fs::write(app_dir.join("index.html"), "x")?;

            Ok(SageAppSnapshot::new(
                "fake-hash",
                app_dir.to_string_lossy().to_string(),
                prepared.manifest.clone(),
            )?)
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

        let granted = SageGrantedPermissionsInput::new(
            [UserBridgeCapability::StoragePersistentWebview],
            [SageNetworkWhitelistEntry::new_unchecked(
                "https",
                "api.example.com",
            )],
        );

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

        let common = installed.common();
        assert_eq!(common.id(), "fake-app");
        assert_eq!(common.origin_id(), "fake-origin");
        assert_eq!(common.name(), "Test App");
        assert_eq!(common.entry_file(), "index.html");
        assert_eq!(common.icon_file(), None);
        assert_eq!(common.storage(), &InstalledSageAppStorage::Unmanaged);
        assert_eq!(installed.source(), &UserSageAppSource::Zip);

        let reread = read_installed_app_by_id(dir.path(), "fake-app").unwrap();
        assert_eq!(reread.common().id(), "fake-app");
        assert_eq!(reread.common().origin_id(), "fake-origin");
    }

    #[tokio::test]
    async fn shared_installer_rejects_unrequested_granted_permission() {
        let dir = tempdir().unwrap();

        let granted = SageGrantedPermissionsInput::new(
            [UserBridgeCapability::StoragePersistentWebview],
            [SageNetworkWhitelistEntry::new_unchecked(
                "https",
                "evil.example.com",
            )],
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

        assert!(
            err.to_string()
                .contains("granted network whitelist entry not requested")
        );
    }

    #[test]
    fn build_installed_app_sets_id_and_origin_id_independently() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("url-abc123");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(app_dir.join("index.html"), "x").unwrap();

        let manifest = sample_manifest();

        let granted_permissions = SageGrantedPermissions::new(
            manifest.permissions(),
            [UserBridgeCapability::StoragePersistentWebview],
            [],
        )
        .unwrap();

        let common = SageAppCommon::new(
            SageAppIdentity::new(
                "url-abc123".to_string(),
                "r123-url-abc123".to_string(),
                app_dir.to_string_lossy().to_string(),
            )
            .unwrap(),
            granted_permissions,
            InstalledSageAppStorage::Unmanaged,
            SageAppSnapshot::new(
                "hash".to_string(),
                app_dir.to_string_lossy().to_string(),
                manifest.clone(),
            )
            .unwrap(),
        )
        .unwrap();
        let app = UserSageApp::new_installed(common, UserSageAppSource::Zip);

        assert_eq!(app.common().id(), "url-abc123");
        assert_eq!(app.common().origin_id(), "r123-url-abc123");
    }
}
