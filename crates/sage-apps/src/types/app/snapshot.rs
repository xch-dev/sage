use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::manifest::SageAppPackageManifest;
use crate::types::normalizers::normalized_non_empty_string;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppSnapshot {
    manifest_hash: String,
    snapshot_dir: String,
    total_bytes: u64,
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
            total_bytes: manifest.total_bytes(),
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

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn snapshot_dir(&self) -> &str {
        &self.snapshot_dir
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
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
