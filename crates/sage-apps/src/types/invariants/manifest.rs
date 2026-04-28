use crate::lifecycle::{
    MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES, validate_manifest_file_path, validate_sha256_hex,
};
use crate::types::SageAppManifestFile;
use crate::types::normalizers::normalized_optional_string;
use anyhow::anyhow;

pub fn normalize_optional_manifest_path(
    path: Option<String>,
    label: &str,
) -> anyhow::Result<Option<String>> {
    let path = normalized_optional_string(path);

    if let Some(path) = &path {
        validate_manifest_file_path(path).map_err(|err| anyhow!("{label} is invalid: {err}"))?;
    }

    Ok(path)
}

pub fn validate_manifest_files(files: &[SageAppManifestFile]) -> anyhow::Result<u64> {
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

pub fn validate_declared_manifest_asset_exists(
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
