use anyhow::{Context, Result as AnyResult};

use crate::{
    MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppPackageManifest, SageAppPackageManifestPreview,
    UserSageAppSource, bytes_sha256_hex, download_bytes_with_limit,
    parse_manifest_header_v0_from_value, validate_network_permissions_for_source,
};

const MAX_URL_MANIFEST_BYTES: u64 = 1024 * 1024;

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

    let bytes = download_bytes_with_limit(manifest_url, MAX_URL_MANIFEST_BYTES)
        .await
        .with_context(|| format!("failed to download manifest from {manifest_url}"))?;

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

            let manifest_header = parse_manifest_header_v0_from_value(value)
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
    validate_network_permissions_for_source(manifest.permissions(), &UserSageAppSource::Zip)?;
    manifest.validate_package_files(package_root)?;

    Ok(manifest)
}

pub fn read_manifest_preview(
    package_root: &std::path::Path,
) -> AnyResult<SageAppPackageManifestPreview> {
    let manifest_path = package_root.join(MANIFEST_FILE_NAME);
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let preview = parse_manifest_preview(&manifest_text, &manifest_path.to_string_lossy())?;

    if let Some(manifest) = preview.full_manifest() {
        manifest.validate_package_files(package_root)?;
    }

    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_preview_preserves_compatibility_header() {
        let manifest = r#"{
            "manifestVersion": 0,
            "name": "Future App",
            "sageVersion": { "min": "1.0.0", "testedMax": "1.2.0" },
            "version": "1.0.0",
            "files": []
        }"#;

        let preview = parse_manifest_preview(manifest, "test manifest").unwrap();

        match preview {
            SageAppPackageManifestPreview::Partial {
                manifest_header,
                parse_error,
            } => {
                assert_eq!(manifest_header.name, "Future App");
                assert_eq!(manifest_header.sage_version.min, "1.0.0");
                assert_eq!(
                    manifest_header.sage_version.tested_max.as_deref(),
                    Some("1.2.0")
                );
                assert!(!parse_error.is_empty());
            }
            SageAppPackageManifestPreview::Full { .. } => {
                panic!("invalid full manifest should use its valid compatibility header")
            }
        }
    }
}
