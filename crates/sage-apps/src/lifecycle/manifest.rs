use std::collections::BTreeSet;

use anyhow::{Context, Result as AnyResult, anyhow};

use crate::{
    lifecycle::limits::{
        MAX_APP_FILE_COUNT, MAX_APP_PATH_LENGTH, MAX_APP_TOTAL_SIZE_BYTES,
    },
    types::{SageAppManifestFile, SageAppPackageManifest},
};
use crate::utils::bytes_sha256_hex;

const MANIFEST_FILE_NAME: &str = "sage-manifest.json";

pub fn manifest_entry_file(manifest: &SageAppPackageManifest) -> &str {
    manifest.entry().unwrap_or("index.html")
}

pub fn manifest_icon_file(manifest: &SageAppPackageManifest) -> &str {
    manifest.icon().unwrap_or("icon.png")
}

pub fn derive_manifest_url(app_url: &str) -> AnyResult<String> {
    let base = reqwest::Url::parse(app_url)
        .with_context(|| format!("invalid app url: {app_url}"))?;

    base.join(MANIFEST_FILE_NAME)
        .map(|url| url.to_string())
        .with_context(|| format!("failed to derive manifest url from app url: {app_url}"))
}

pub async fn fetch_url_manifest(
    manifest_url: &str,
) -> AnyResult<(SageAppPackageManifest, String)> {
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

    let manifest: SageAppPackageManifest = serde_json::from_str(manifest_text)
        .with_context(|| format!("failed to parse manifest json from {manifest_url}"))?;

    Ok((manifest, manifest_hash))
}

pub fn read_manifest(package_root: &std::path::Path) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = package_root.join(MANIFEST_FILE_NAME);
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: SageAppPackageManifest =
        serde_json::from_str(&manifest_text).context("failed to parse manifest")?;
    Ok(manifest)
}

pub fn validate_manifest_file_path(path: &str) -> AnyResult<()> {
    if path.is_empty() {
        return Err(anyhow!("manifest file path cannot be empty"));
    }

    if path.len() > MAX_APP_PATH_LENGTH {
        return Err(anyhow!(
            "manifest file path exceeds max length {}: {}",
            MAX_APP_PATH_LENGTH,
            path
        ));
    }

    if path.starts_with('/') || path.starts_with('\\') {
        return Err(anyhow!("manifest file path must be relative: {}", path));
    }

    if path.contains('\\') {
        return Err(anyhow!(
            "manifest file path must use forward slashes: {}",
            path
        ));
    }

    if path
        .split('/')
        .any(|part| part == "." || part == ".." || part.is_empty())
    {
        return Err(anyhow!("manifest file path is invalid: {}", path));
    }

    Ok(())
}

pub fn validate_sha256_hex(value: &str) -> AnyResult<()> {
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("invalid sha256 hex: {}", value));
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
        validate_manifest_file_path(&file.path)?;
        validate_sha256_hex(&file.sha256)?;

        if !seen.insert(file.path.clone()) {
            return Err(anyhow!("duplicate manifest file path: {}", file.path));
        }

        total = total
            .checked_add(file.size)
            .ok_or_else(|| anyhow!("manifest total size overflow"))?;
    }

    if total > MAX_APP_TOTAL_SIZE_BYTES {
        return Err(anyhow!(
            "manifest total size {} exceeds limit {}",
            total,
            MAX_APP_TOTAL_SIZE_BYTES
        ));
    }

    Ok(total)
}
