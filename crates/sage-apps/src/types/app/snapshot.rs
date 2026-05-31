use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde::{Deserialize, Deserializer};

use crate::{SageAppPackageManifest, normalized_non_empty_string};

#[derive(Debug, Clone, Serialize)]
pub struct SageAppSnapshot {
    manifest_hash: String,
    snapshot_dir: String,
    manifest: SageAppPackageManifest,
}

impl SageAppSnapshot {
    pub fn new(
        manifest_hash: impl Into<String>,
        snapshot_dir: impl Into<String>,
        manifest: SageAppPackageManifest,
    ) -> anyhow::Result<Self> {
        let manifest_hash = normalized_non_empty_string(manifest_hash, "snapshot manifest hash")?;
        let snapshot_dir = normalized_non_empty_string(snapshot_dir, "snapshot directory")?;

        let snapshot = Self {
            manifest_hash,
            snapshot_dir,
            manifest,
        };

        snapshot.validate_files_exist_internal()?;

        Ok(snapshot)
    }

    pub fn new_builtin_system(
        app_id: &str,
        snapshot_dir: impl Into<String>,
        manifest: SageAppPackageManifest,
    ) -> anyhow::Result<Self> {
        Self::new(format!("builtin-system:{app_id}"), snapshot_dir, manifest)
    }

    pub fn resolve_file_path(&self, request_path: &str) -> anyhow::Result<PathBuf> {
        let normalized = if request_path.is_empty() || request_path == "/" {
            self.manifest().entry()
        } else {
            request_path.trim_start_matches('/')
        };

        let relative = Path::new(normalized);

        if relative.is_absolute() {
            anyhow::bail!("snapshot path must be relative");
        }

        for component in relative.components() {
            match component {
                Component::Normal(_) => {}
                _ => anyhow::bail!("invalid snapshot path component in {request_path}"),
            }
        }

        let root = Path::new(self.snapshot_dir());
        let path = root.join(relative);

        if !path.is_file() {
            anyhow::bail!("snapshot file not found: {request_path}");
        }

        let canonical_root = root.canonicalize()?;
        let canonical_path = path.canonicalize()?;

        if !canonical_path.starts_with(&canonical_root) {
            anyhow::bail!("snapshot path escapes root: {request_path}");
        }

        Ok(canonical_path)
    }

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn snapshot_dir(&self) -> &str {
        &self.snapshot_dir
    }

    pub fn manifest(&self) -> &SageAppPackageManifest {
        &self.manifest
    }

    pub fn file_path(&self, path: &str) -> PathBuf {
        Path::new(&self.snapshot_dir).join(path)
    }

    fn validate_files_exist_internal(&self) -> anyhow::Result<()> {
        for file in self.manifest.files() {
            let path = self.file_path(file.path());

            if !path.is_file() {
                anyhow::bail!("snapshot file does not exist: {}", path.display());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SageAppSnapshotDeserialize {
    manifest_hash: String,
    snapshot_dir: String,
    manifest: SageAppPackageManifest,
}

impl<'de> Deserialize<'de> for SageAppSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = SageAppSnapshotDeserialize::deserialize(deserializer)?;

        SageAppSnapshot::new(raw.manifest_hash, raw.snapshot_dir, raw.manifest)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{
        SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion,
        SageAppPackageManifestParts, SageRequestedPermissions,
    };

    fn manifest_file(path: &str) -> SageAppManifestFile {
        SageAppManifestFile::new(path, "a".repeat(64), 1).unwrap()
    }

    fn manifest() -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(0),
            name: "test app".to_string(),
            icon: None,
            sage_version: SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![manifest_file("index.html"), manifest_file("nested/file.js")],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn snapshot() -> (SageAppSnapshot, TempDir) {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "x").unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("nested/file.js"), "x").unwrap();

        let snapshot =
            SageAppSnapshot::new("hash", dir.path().to_string_lossy(), manifest()).unwrap();

        (snapshot, dir)
    }

    #[test]
    fn resolve_file_path_defaults_empty_and_root_to_manifest_entry() {
        let (snapshot, _dir) = snapshot();
        let entry = snapshot.file_path("index.html").canonicalize().unwrap();

        assert_eq!(snapshot.resolve_file_path("").unwrap(), entry);
        assert_eq!(snapshot.resolve_file_path("/").unwrap(), entry);
    }

    #[test]
    fn resolve_file_path_accepts_nested_snapshot_file() {
        let (snapshot, _dir) = snapshot();
        let nested = snapshot.file_path("nested/file.js").canonicalize().unwrap();

        assert_eq!(
            snapshot.resolve_file_path("/nested/file.js").unwrap(),
            nested
        );
    }

    #[test]
    fn resolve_file_path_rejects_traversal_components() {
        let (snapshot, _dir) = snapshot();

        for path in ["../secret.txt", "/../secret.txt", "nested/../index.html"] {
            assert!(
                snapshot.resolve_file_path(path).is_err(),
                "expected {path} to be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn resolve_file_path_rejects_symlink_that_escapes_snapshot_root() {
        let (snapshot, dir) = snapshot();
        let external = dir.path().parent().unwrap().join("external-secret.txt");
        fs::write(&external, "secret").unwrap();
        std::os::unix::fs::symlink(&external, dir.path().join("nested/link.txt")).unwrap();

        let err = snapshot
            .resolve_file_path("nested/link.txt")
            .expect_err("escaping symlink must be rejected");

        assert!(
            err.to_string().contains("escapes root"),
            "unexpected error: {err}"
        );
    }
}
