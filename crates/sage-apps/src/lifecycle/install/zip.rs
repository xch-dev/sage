use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result as AnyResult;
use uuid::Uuid;

use crate::{
    AppInstallSource, SageAppPackageManifest, SageAppSnapshot, UserSageAppSource,
    detect_package_root, prepare_zip_snapshot, read_manifest, slugify_app_name, unzip_to_dir,
};

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
}

impl AppInstallSource for ZipInstallSource {
    type PreparedArtifact = PreparedZipInstall;

    async fn prepare(&self) -> AnyResult<Self::PreparedArtifact> {
        if self.unpack_dir.exists() {
            fs::remove_dir_all(&self.unpack_dir)?;
        }

        unzip_to_dir(Path::new(&self.zip_path), &self.unpack_dir)?;

        let package_root = detect_package_root(&self.unpack_dir)?;
        let manifest = read_manifest(&package_root)?;

        Ok(PreparedZipInstall {
            package_root,
            manifest,
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::PreparedArtifact) -> &'a SageAppPackageManifest {
        &prepared.manifest
    }

    fn source(&self, _prepared: &Self::PreparedArtifact) -> UserSageAppSource {
        UserSageAppSource::Zip
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<(String, PathBuf)> {
        let app_id = generate_zip_app_id(prepared.manifest.name());
        let app_dir = root.join(&app_id);

        Ok((app_id, app_dir))
    }

    async fn create_snapshot(
        &self,
        snapshot_dir: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<SageAppSnapshot> {
        prepare_zip_snapshot(&prepared.package_root, snapshot_dir, &prepared.manifest)
    }
}

pub fn generate_zip_app_id(name: &str) -> String {
    format!("{}-{}", slugify_app_name(name), Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{SageAppManifestFile, SageAppPackageManifestParts, SageRequestedPermissions};

    fn sample_manifest(name: &str) -> SageAppPackageManifest {
        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: name.to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 1).unwrap()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    #[test]
    fn generate_zip_app_id_uses_slug_and_uuid() {
        let app_id = generate_zip_app_id("My Test App!");

        assert!(app_id.starts_with("my-test-app-"));
        assert!(app_id.len() > "my-test-app-".len());
    }

    #[test]
    fn zip_resolve_target_always_creates_new_install_target() {
        let dir = tempdir().unwrap();

        let source = ZipInstallSource {
            zip_path: "unused.zip".into(),
            unpack_dir: dir.path().join(".tmp-unused"),
        };

        let prepared = PreparedZipInstall {
            package_root: dir.path().to_path_buf(),
            manifest: sample_manifest("Test App"),
        };

        let (app_id, app_dir) = source
            .resolve_target(dir.path(), dir.path(), &prepared)
            .unwrap();

        assert!(app_id.starts_with("test-app-"));
        assert_eq!(app_dir, dir.path().join(&app_id));
    }

    #[test]
    fn zip_resolve_target_does_not_reuse_existing_app_with_same_name() {
        let dir = tempdir().unwrap();

        let source = ZipInstallSource {
            zip_path: "unused.zip".into(),
            unpack_dir: dir.path().join(".tmp-unused"),
        };

        let prepared = PreparedZipInstall {
            package_root: dir.path().to_path_buf(),
            manifest: sample_manifest("Test App"),
        };

        let (first_app_id, _) = source
            .resolve_target(dir.path(), dir.path(), &prepared)
            .unwrap();

        let (second_app_id, _) = source
            .resolve_target(dir.path(), dir.path(), &prepared)
            .unwrap();

        assert_ne!(first_app_id, second_app_id);
        assert!(first_app_id.starts_with("test-app-"));
        assert!(second_app_id.starts_with("test-app-"));
    }
}
