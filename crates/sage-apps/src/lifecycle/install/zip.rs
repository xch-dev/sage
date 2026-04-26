use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use uuid::Uuid;

use super::AppInstallSource;
use crate::lifecycle::{
    detect_package_root, list_installed_apps_internal, prepare_zip_snapshot, read_manifest,
    unzip_to_dir, validate_package_structure,
};
use crate::permissions::normalize_and_validate_requested_permissions;
use crate::types::{
    ListedSageApp, SageAppPackageManifest, SageAppSnapshot, UserSageApp, UserSageAppSource,
};
use crate::utils::slugify_app_name;

#[derive(Debug, Clone)]
pub struct ZipInstallSource {
    pub zip_path: String,
    pub unpack_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PreparedZipInstall {
    pub package_root: PathBuf,
    pub manifest: SageAppPackageManifest,
}

impl ZipInstallSource {
    pub fn new(root: &Path, zip_path: String) -> Self {
        Self {
            zip_path,
            unpack_dir: root.join(format!(".tmp-{}", Uuid::new_v4())),
        }
    }

    pub fn cleanup(&self) {
        if self.unpack_dir.exists() {
            let _ = fs::remove_dir_all(&self.unpack_dir);
        }
    }
}

#[async_trait]
impl AppInstallSource for ZipInstallSource {
    type Prepared = PreparedZipInstall;

    async fn prepare(&self) -> AnyResult<Self::Prepared> {
        if self.unpack_dir.exists() {
            fs::remove_dir_all(&self.unpack_dir)?;
        }

        unzip_to_dir(Path::new(&self.zip_path), &self.unpack_dir)?;

        let package_root = detect_package_root(&self.unpack_dir)?;
        validate_package_structure(&package_root)?;

        let manifest = normalize_manifest_permissions(read_manifest(&package_root)?)?;

        Ok(PreparedZipInstall {
            package_root,
            manifest,
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::Prepared) -> &'a SageAppPackageManifest {
        &prepared.manifest
    }

    fn source(&self, _prepared: &Self::Prepared) -> UserSageAppSource {
        UserSageAppSource::Zip
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
        resolve_zip_install_target(root, &prepared.manifest.name)
    }

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<SageAppSnapshot> {
        prepare_zip_snapshot(&prepared.package_root, app_dir, &prepared.manifest)
    }
}

pub fn generate_zip_app_id(name: &str) -> String {
    format!("{}-{}", slugify_app_name(name), Uuid::new_v4())
}

pub fn resolve_zip_install_target(
    root: &Path,
    app_name: &str,
) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
    if let Some(existing) = find_existing_installed_app_by_name(root, app_name)? {
        let app_dir = Path::new(&existing.common.app_dir).to_path_buf();
        return Ok((existing.common.id.clone(), app_dir, Some(existing)));
    }

    let app_id = generate_zip_app_id(app_name);
    Ok((app_id.clone(), root.join(&app_id), None))
}

fn find_existing_installed_app_by_name(
    root: &Path,
    app_name: &str,
) -> AnyResult<Option<UserSageApp>> {
    Ok(list_installed_apps_internal(root)?
        .into_iter()
        .find_map(|app| match app {
            ListedSageApp::User(installed) if installed.common.name == app_name => Some(installed),
            _ => None,
        }))
}

fn normalize_manifest_permissions(
    mut manifest: SageAppPackageManifest,
) -> AnyResult<SageAppPackageManifest> {
    manifest.permissions = normalize_and_validate_requested_permissions(&manifest.permissions)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::write_installed_app_metadata;
    use crate::types::{
        InstalledSageAppStorage, SageAppFlags, SageAppManifestFile,
        SageGrantedNetworkPermissions, SageGrantedPermissions, SageRequestedCapabilities,
        SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions,
    };
    use tempfile::tempdir;

    fn sample_manifest_named(name: &str) -> SageAppPackageManifest {
        SageAppPackageManifest {
            name: name.into(),
            version: "1.0.0".into(),
            permissions: SageRequestedPermissions {
                network: SageRequestedNetworkPermissions {
                    whitelist: SageRequestedNetworkWhitelist {
                        required: vec![],
                        optional: vec![],
                    },
                },
                capabilities: SageRequestedCapabilities {
                    required: vec![],
                    optional: vec![],
                },
            },
            files: vec![SageAppManifestFile {
                path: "index.html".into(),
                sha256: "a".repeat(64),
                size: 123,
            }],
            entry: Some("index.html".into()),
            icon: Some("icon.png".into()),
            author: None,
            donation: None,
        }
    }

    #[test]
    fn generate_zip_app_id_uses_slug_and_uuid() {
        let app_id = generate_zip_app_id("My Test App!");

        assert!(app_id.starts_with("my-test-app-"));
        assert!(app_id.len() > "my-test-app-".len());
    }

    #[test]
    fn resolve_zip_install_target_creates_new_target_when_no_existing_app() {
        let dir = tempdir().unwrap();

        let (app_id, app_dir, existing) =
            resolve_zip_install_target(dir.path(), "Test App").unwrap();

        assert!(existing.is_none());
        assert!(app_id.starts_with("test-app-"));
        assert_eq!(app_dir, dir.path().join(&app_id));
    }

    #[test]
    fn resolve_zip_install_target_reuses_existing_app_with_same_name() {
        let dir = tempdir().unwrap();
        let app_id = "existing-app";
        let app_dir = dir.path().join(app_id);
        fs::create_dir_all(&app_dir).unwrap();

        let manifest = sample_manifest_named("Test App");

        let installed = super::super::build_installed_app(
            app_id.into(),
            app_id.into(),
            &app_dir,
            &manifest,
            SageGrantedPermissions {
                capabilities: vec![UserBridgeCapability::PersistentStorage],
                network: SageGrantedNetworkPermissions { whitelist: vec![] },
            },
            SageAppFlags::default(),
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::Zip,
            SageAppSnapshot {
                manifest_hash: "hash".into(),
                snapshot_dir: app_dir.to_string_lossy().to_string(),
                total_bytes: 123,
                manifest: manifest.clone(),
            },
        );

        write_installed_app_metadata(&installed, &app_dir).unwrap();

        let (resolved_id, resolved_dir, existing) =
            resolve_zip_install_target(dir.path(), "Test App").unwrap();

        assert_eq!(resolved_id, app_id);
        assert_eq!(resolved_dir, app_dir);
        assert!(existing.is_some());
        assert_eq!(existing.unwrap().common.id, app_id);
    }

    #[test]
    fn zip_origin_id_defaults_to_app_id() {
        let source = ZipInstallSource {
            zip_path: "unused.zip".into(),
            unpack_dir: std::env::temp_dir(),
        };

        let origin = source
            .origin_id(Path::new("/unused"), "zip-app-123", None)
            .unwrap();

        assert_eq!(origin, "zip-app-123");
    }
}
