use std::path::PathBuf;

use anyhow::anyhow;

use crate::lifecycle::{
    MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES, validate_manifest_file_path, validate_sha256_hex,
};
use super::app::SageAppFlags;
use crate::types::SageAppSnapshot;
use crate::types::manifest::SageAppManifestFile;
use crate::types::normalizers::{normalized_non_empty_string, normalized_optional_string};

pub(super) struct NormalizedAppIdentity {
    pub id: String,
    pub origin_id: String,
    pub app_dir: String,
}

pub(super) fn normalize_app_identity(
    id: String,
    origin_id: String,
    app_dir: String,
) -> anyhow::Result<NormalizedAppIdentity> {
    Ok(NormalizedAppIdentity {
        id: normalized_non_empty_string(id, "app id")?,
        origin_id: normalized_non_empty_string(origin_id, "app origin id")?,
        app_dir: normalized_non_empty_string(app_dir, "app directory")?,
    })
}

pub(super) fn normalize_optional_manifest_path(
    path: Option<String>,
    label: &str,
) -> anyhow::Result<Option<String>> {
    let path = normalized_optional_string(path);

    if let Some(path) = &path {
        validate_manifest_file_path(path).map_err(|err| anyhow!("{label} is invalid: {err}"))?;
    }

    Ok(path)
}

pub(super) fn validate_manifest_files(files: &[SageAppManifestFile]) -> anyhow::Result<u64> {
    if files.is_empty() {
        anyhow::bail!("manifest files cannot be empty");
    }

    if files.len() > MAX_APP_FILE_COUNT {
        anyhow::bail!(
            "manifest file count {} exceeds limit {}",
            files.len(),
            MAX_APP_FILE_COUNT
        );
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut total: u64 = 0;

    for file in files {
        validate_manifest_file_path(file.path())?;
        validate_sha256_hex(file.sha256())?;

        if !seen.insert(file.path().to_string()) {
            anyhow::bail!("duplicate manifest file path: {}", file.path());
        }

        total = total
            .checked_add(file.size())
            .ok_or_else(|| anyhow!("manifest total size overflow"))?;
    }

    if total > MAX_APP_TOTAL_SIZE_BYTES {
        anyhow::bail!("manifest total size {total} exceeds limit {MAX_APP_TOTAL_SIZE_BYTES}");
    }

    Ok(total)
}

pub(super) fn validate_declared_manifest_asset_exists(
    path: Option<&str>,
    files: &[SageAppManifestFile],
    label: &str,
) -> anyhow::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    if !files.iter().any(|file| file.path() == path) {
        anyhow::bail!("manifest {label} file is not listed in files: {path}");
    }

    Ok(())
}

pub(super) fn validate_app_flags_policy(flags: SageAppFlags) -> anyhow::Result<()> {
    if flags.has_external_access() && flags.has_secret_access() {
        anyhow::bail!(
            "app permissions cannot include both external access and sensitive secret access"
        );
    }

    if flags.has_external_access() && flags.storage_may_contain_secrets() {
        anyhow::bail!(
            "app permissions cannot include external access while storage may contain secrets"
        );
    }

    Ok(())
}

pub(super) fn validate_snapshot_entry_and_icon_exist(
    snapshot: &SageAppSnapshot,
    entry_file: &str,
    icon_file: Option<&str>,
    label: &str,
) -> anyhow::Result<()> {
    let entry_file = snapshot.file_path(entry_file);

    if !entry_file.is_file() {
        anyhow::bail!(
            "{label} entry file does not exist: {}",
            entry_file.display()
        );
    }

    if let Some(icon_file) = icon_file {
        let icon_file: PathBuf = snapshot.file_path(icon_file);

        if !icon_file.is_file() {
            anyhow::bail!("{label} icon file does not exist: {}", icon_file.display());
        }
    }

    Ok(())
}
