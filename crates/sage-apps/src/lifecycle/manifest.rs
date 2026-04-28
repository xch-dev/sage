use std::collections::BTreeSet;

use anyhow::{Context, Result as AnyResult, anyhow};

use crate::utils::bytes_sha256_hex;
use crate::{
    lifecycle::limits::{MAX_APP_FILE_COUNT, MAX_APP_PATH_LENGTH, MAX_APP_TOTAL_SIZE_BYTES},
    types::{SageAppManifestFile, SageAppPackageManifest},
};

const MANIFEST_FILE_NAME: &str = "sage-manifest.json";

pub fn manifest_entry_file(manifest: &SageAppPackageManifest) -> &str {
    manifest.entry().unwrap_or("index.html")
}

pub fn manifest_icon_file(manifest: &SageAppPackageManifest) -> Option<&str> {
    manifest.icon()
}

pub fn derive_manifest_url(app_url: &str) -> AnyResult<String> {
    let base =
        reqwest::Url::parse(app_url).with_context(|| format!("invalid app url: {app_url}"))?;

    base.join(MANIFEST_FILE_NAME)
        .map(|url| url.to_string())
        .with_context(|| format!("failed to derive manifest url from app url: {app_url}"))
}

pub async fn fetch_url_manifest(manifest_url: &str) -> AnyResult<(SageAppPackageManifest, String)> {
    let response = reqwest::get(manifest_url)
        .await
        .with_context(|| format!("failed to GET manifest url {manifest_url}"))?
        .error_for_status()
        .with_context(|| format!("manifest request failed for {manifest_url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read manifest response body from {manifest_url}"))?;

    let manifest_hash = bytes_sha256_hex(&bytes);

    let manifest_text = std::str::from_utf8(&bytes)
        .with_context(|| format!("manifest is not valid UTF-8: {manifest_url}"))?;

    let mut deserializer = serde_json::Deserializer::from_str(manifest_text);
    let manifest: SageAppPackageManifest = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|err| {
            anyhow::anyhow!(
            "failed to parse manifest json from {manifest_url} at {}: {}",
            err.path(),
            err.inner()
        )
        })?;

    Ok((manifest, manifest_hash))
}

pub fn read_manifest(package_root: &std::path::Path) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = package_root.join(MANIFEST_FILE_NAME);
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut deserializer = serde_json::Deserializer::from_str(&manifest_text);
    let manifest: SageAppPackageManifest = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|err| {
            anyhow::anyhow!(
            "failed to parse manifest {} at {}: {}",
            manifest_path.display(),
            err.path(),
            err.inner()
        )
        })?;
    Ok(manifest)
}

pub fn validate_manifest_file_path(path: &str) -> AnyResult<()> {
    if path.is_empty() {
        return Err(anyhow!("manifest file path cannot be empty"));
    }

    if path.len() > MAX_APP_PATH_LENGTH {
        return Err(anyhow!(
            "manifest file path exceeds max length {MAX_APP_PATH_LENGTH}: {path}"
        ));
    }

    if path.starts_with('/') || path.starts_with('\\') {
        return Err(anyhow!("manifest file path must be relative: {path}"));
    }

    if path.contains('\\') {
        return Err(anyhow!(
            "manifest file path must use forward slashes: {path}"
        ));
    }

    if path
        .split('/')
        .any(|part| part == "." || part == ".." || part.is_empty())
    {
        return Err(anyhow!("manifest file path is invalid: {path}"));
    }

    Ok(())
}

pub fn validate_sha256_hex(value: &str) -> AnyResult<()> {
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("invalid sha256 hex: {value}"));
    }

    Ok(())
}

pub fn validate_manifest_files(files: &[SageAppManifestFile]) -> AnyResult<u64> {
    if files.is_empty() {
        return Err(anyhow!("manifest files cannot be empty"));
    }

    if files.len() > MAX_APP_FILE_COUNT {
        return Err(anyhow!(
            "manifest file count {} exceeds limit {}",
            files.len(),
            MAX_APP_FILE_COUNT
        ));
    }

    let mut seen = BTreeSet::new();
    let mut total: u64 = 0;

    for file in files {
        validate_manifest_file_path(file.path())?;
        validate_sha256_hex(file.sha256())?;

        if !seen.insert(file.path().to_string()) {
            return Err(anyhow!("duplicate manifest file path: {}", file.path()));
        }

        total = total
            .checked_add(file.size())
            .ok_or_else(|| anyhow!("manifest total size overflow"))?;
    }

    if total > MAX_APP_TOTAL_SIZE_BYTES {
        return Err(anyhow!(
            "manifest total size {total} exceeds limit {MAX_APP_TOTAL_SIZE_BYTES}"
        ));
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::limits::{MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES};
    use crate::types::{
        SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions,
    };

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(
            path.to_string(),
            "a".repeat(64),
            size,
        ).unwrap()
    }

    fn sample_file() -> SageAppManifestFile {
        sample_manifest_file("index.html", 123)
    }

    fn entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn requested_permissions() -> SageRequestedPermissions {
        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [entry("https", "required.example.com")],
                [entry("wss", "optional.example.com")],
            ),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::WalletSendXch],
                [UserBridgeCapability::PersistentStorage, UserBridgeCapability::WalletGetSecretKey],
            ),
        )
            .unwrap()
    }

    fn sample_manifest_with(
        entry_file: Option<String>,
        icon_file: Option<String>,
    ) -> SageAppPackageManifest {
        let mut files = vec![sample_manifest_file("index.html", 1)];

        if let Some(entry_file) = &entry_file
            && entry_file != "index.html"
        {
            files.push(sample_manifest_file(entry_file, 1));
        }

        if let Some(icon_file) = &icon_file {
            files.push(sample_manifest_file(icon_file, 1));
        }

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: requested_permissions(),
            files,
            entry: entry_file,
            icon: icon_file,
            author: None,
            donation: None,
        })
            .unwrap()
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

    #[test]
    fn manifest_rejects_blank_name() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "   ".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
            .unwrap_err();

        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn manifest_rejects_blank_version() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test".to_string(),
            version: "   ".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
            .unwrap_err();

        assert!(err.to_string().contains("version cannot be empty"));
    }

    #[test]
    fn manifest_total_size_is_computed() {
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("dist/index.html", 123)],
            entry: Some("dist/index.html".to_string()),
            icon: None,
            author: None,
            donation: None,
        })
            .unwrap();

        assert_eq!(manifest.total_bytes(), 123);
    }

    #[test]
    fn manifest_entry_file_uses_explicit_entry() {
        let manifest = sample_manifest_with(
            Some("entry.html".to_string()),
            Some("icon.svg".to_string()),
        );

        assert_eq!(manifest_entry_file(&manifest), "entry.html");
    }

    #[test]
    fn manifest_entry_file_defaults_to_index_html() {
        let manifest = sample_manifest_with(None, Some("icon.svg".to_string()));
        assert_eq!(manifest_entry_file(&manifest), "index.html");
    }

    #[test]
    fn manifest_icon_file_uses_explicit_icon() {
        let manifest = sample_manifest_with(
            Some("entry.html".to_string()),
            Some("icon.svg".to_string()),
        );

        assert_eq!(manifest_icon_file(&manifest).unwrap(), "icon.svg");
    }

    #[test]
    fn manifest_icon_file_defaults_to_none() {
        let manifest = sample_manifest_with(Some("entry.html".to_string()), None);
        assert_eq!(manifest_icon_file(&manifest), None);
    }
}
