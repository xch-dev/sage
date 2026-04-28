use anyhow::{Context, Result as AnyResult};

use crate::types::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppPackageManifest};
use crate::utils::bytes_sha256_hex;

pub async fn fetch_url_manifest(
    manifest_url: &SageAppManifestUrl,
) -> AnyResult<(SageAppPackageManifest, String)> {
    let manifest_url = manifest_url.as_str();

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
