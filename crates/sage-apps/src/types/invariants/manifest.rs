use std::fs;
use std::path::Path;

use anyhow::{Context, anyhow};

use crate::types::SageAppManifestFile;
use crate::types::normalized_optional_string;
use crate::utils::bytes_sha256_hex;

pub const MAX_APP_FILE_COUNT: usize = 2000;
pub const MAX_APP_TOTAL_SIZE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_APP_PATH_LENGTH: usize = 512;

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

pub fn validate_manifest_file_path(path: &str) -> anyhow::Result<()> {
    if path.is_empty() {
        anyhow::bail!("manifest file path cannot be empty");
    }

    if path.len() > MAX_APP_PATH_LENGTH {
        anyhow::bail!("manifest file path exceeds max length {MAX_APP_PATH_LENGTH}: {path}");
    }

    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!("manifest file path must be relative: {path}");
    }

    if path.contains('\\') {
        anyhow::bail!("manifest file path must use forward slashes: {path}");
    }

    if path
        .split('/')
        .any(|part| part == "." || part == ".." || part.is_empty())
    {
        anyhow::bail!("manifest file path is invalid: {path}");
    }

    Ok(())
}

pub fn validate_sha256_hex(value: &str) -> anyhow::Result<()> {
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("invalid sha256 hex: {value}");
    }

    Ok(())
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

pub fn validate_package_files_match_manifest(
    package_root: &Path,
    files: &[SageAppManifestFile],
) -> anyhow::Result<()> {
    for file in files {
        let relative_path = file.path();
        let path = package_root.join(relative_path);

        if !path.is_file() {
            anyhow::bail!("manifest file missing from package: {relative_path}");
        }

        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read package file {}", path.display()))?;

        let actual_hash = bytes_sha256_hex(&bytes);
        if actual_hash != file.sha256() {
            anyhow::bail!("sha256 mismatch for {relative_path}");
        }

        let actual_size = u64::try_from(bytes.len()).context("file too large")?;
        if actual_size != file.size() {
            anyhow::bail!("size mismatch for {relative_path}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(path.to_string(), "a".repeat(64), size).unwrap()
    }

    #[test]
    fn validate_manifest_file_path_accepts_normal_relative_path() {
        validate_manifest_file_path("dist/index.html").unwrap();
    }

    #[test]
    fn validate_manifest_file_path_rejects_absolute_path() {
        assert!(validate_manifest_file_path("/etc/passwd").is_err());
    }

    #[test]
    fn validate_manifest_file_path_rejects_parent_traversal() {
        assert!(validate_manifest_file_path("../secret.txt").is_err());
    }

    #[test]
    fn validate_manifest_file_path_rejects_current_dir_segment() {
        assert!(validate_manifest_file_path("./index.html").is_err());
        assert!(validate_manifest_file_path("dist/./index.html").is_err());
    }

    #[test]
    fn validate_manifest_file_path_rejects_empty_segment() {
        assert!(validate_manifest_file_path("dist//index.html").is_err());
    }

    #[test]
    fn validate_manifest_file_path_rejects_backslashes() {
        assert!(validate_manifest_file_path(r"dist\index.html").is_err());
    }

    #[test]
    fn validate_sha256_hex_accepts_valid_hash() {
        validate_sha256_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
    }

    #[test]
    fn validate_sha256_hex_rejects_invalid_hash() {
        assert!(validate_sha256_hex("not-a-sha").is_err());
    }

    #[test]
    fn validate_manifest_files_rejects_empty_list() {
        let err = validate_manifest_files(&[]).unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn validate_manifest_files_rejects_duplicate_paths() {
        let files = vec![
            sample_manifest_file("dist/index.html", 1),
            sample_manifest_file("dist/index.html", 2),
        ];

        let err = validate_manifest_files(&files).unwrap_err();
        assert!(err.to_string().contains("duplicate manifest file path"));
    }

    #[test]
    fn validate_manifest_files_rejects_invalid_nested_path() {
        let err = SageAppManifestFile::new("dist//index.html", "a".repeat(64), 1).unwrap_err();
        assert!(err.to_string().contains("manifest file path is invalid"));
    }

    #[test]
    fn validate_manifest_files_rejects_file_count_over_limit() {
        let files: Vec<_> = (0..=MAX_APP_FILE_COUNT)
            .map(|i| sample_manifest_file(&format!("dist/file-{i}.txt"), 1))
            .collect();

        let err = validate_manifest_files(&files).unwrap_err();
        assert!(err.to_string().contains("exceeds limit"));
    }

    #[test]
    fn validate_manifest_files_rejects_total_size_over_limit() {
        let files = vec![
            sample_manifest_file("dist/a.bin", MAX_APP_TOTAL_SIZE_BYTES),
            sample_manifest_file("dist/b.bin", 1),
        ];

        let err = validate_manifest_files(&files).unwrap_err();
        assert!(err.to_string().contains("manifest total size"));
        assert!(err.to_string().contains("exceeds limit"));
    }

    #[test]
    fn validate_manifest_files_returns_total_size_when_valid() {
        let files = vec![
            sample_manifest_file("dist/index.html", 100),
            sample_manifest_file("dist/icon.png", 23),
        ];

        let total = validate_manifest_files(&files).unwrap();
        assert_eq!(total, 123);
    }
}
