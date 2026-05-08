use anyhow::{Context, Result as AnyResult};

use crate::types::{
    MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppPackageManifest, SageAppPackageManifestPreview,
};
use crate::utils::bytes_sha256_hex;

pub async fn fetch_url_manifest(
    manifest_url: &SageAppManifestUrl,
) -> AnyResult<(SageAppPackageManifest, String)> {
    let (preview, hash) = fetch_url_manifest_preview(manifest_url).await?;

    match preview {
        SageAppPackageManifestPreview::Full { manifest } => Ok((manifest, hash)),
        SageAppPackageManifestPreview::Partial { parse_error, .. } => Err(anyhow::anyhow!(
            "failed to parse manifest json: {parse_error}"
        )),
    }
}

pub async fn fetch_url_manifest_preview(
    manifest_url: &SageAppManifestUrl,
) -> AnyResult<(SageAppPackageManifestPreview, String)> {
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

    Ok((
        parse_manifest_preview(manifest_text, manifest_url)?,
        manifest_hash,
    ))
}

pub fn parse_manifest_preview(
    manifest_text: &str,
    source_label: &str,
) -> AnyResult<SageAppPackageManifestPreview> {
    let mut deserializer = serde_json::Deserializer::from_str(manifest_text);

    match serde_path_to_error::deserialize::<_, SageAppPackageManifest>(&mut deserializer) {
        Ok(manifest) => Ok(SageAppPackageManifestPreview::Full { manifest }),

        Err(full_err) => {
            let parse_error = format!("at {}: {}", full_err.path(), full_err.inner());

            let value: serde_json::Value = serde_json::from_str(manifest_text)
                .with_context(|| format!("failed to parse manifest JSON from {source_label}"))?;

            let manifest_header = crate::types::parse_manifest_header_v0_from_value(value)
                .with_context(|| {
                    format!(
                        "failed to parse fallback manifest header from {source_label}; full parse failed {parse_error}"
                    )
                })?;

            Ok(SageAppPackageManifestPreview::Partial {
                manifest_header,
                parse_error,
            })
        }
    }
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
    manifest.validate_package_files(package_root)?;

    Ok(manifest)
}
