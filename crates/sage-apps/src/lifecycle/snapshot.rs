use crate::types::{SageAppPackageManifest, SageAppSnapshot, SageAppUrl};
use crate::utils::bytes_sha256_hex;
use anyhow::{Context, Result as AnyResult, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) async fn download_bytes_with_limit(url: &str, max_bytes: u64) -> AnyResult<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("request failed for {url}"))?;

    if response
        .content_length()
        .is_some_and(|content_length| content_length > max_bytes)
    {
        anyhow::bail!("response body from {url} exceeds maximum size {max_bytes}");
    }

    let capacity = usize::try_from(max_bytes.min(1024 * 1024)).unwrap_or(1024 * 1024);
    let mut bytes = Vec::with_capacity(capacity);
    let mut response = response;
    let mut received = 0u64;

    while let Some(chunk) = response
        .chunk()
        .await
        .with_context(|| format!("failed to read response body from {url}"))?
    {
        let chunk_len = u64::try_from(chunk.len()).context("response chunk too large")?;

        received = received
            .checked_add(chunk_len)
            .context("response body size overflow")?;

        if received > max_bytes {
            anyhow::bail!("response body from {url} exceeds maximum size {max_bytes}");
        }

        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
}

async fn download_exact_bytes(url: &str, expected_size: u64) -> AnyResult<Vec<u8>> {
    let bytes = download_bytes_with_limit(url, expected_size).await?;

    if u64::try_from(bytes.len()).context("response body too large")? != expected_size {
        anyhow::bail!("response body from {url} did not match expected size {expected_size}");
    }

    Ok(bytes)
}

fn write_file(path: &Path, bytes: &[u8]) -> AnyResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

pub async fn download_url_snapshot(
    snapshot_dir: &Path,
    app_url: &SageAppUrl,
    manifest: &SageAppPackageManifest,
    manifest_hash: &str,
) -> AnyResult<SageAppSnapshot> {
    if snapshot_dir.exists() {
        fs::remove_dir_all(snapshot_dir).with_context(|| {
            format!(
                "failed to remove existing snapshot dir {}",
                snapshot_dir.display()
            )
        })?;
    }

    fs::create_dir_all(snapshot_dir)
        .with_context(|| format!("failed to create snapshot dir {}", snapshot_dir.display()))?;

    for file in manifest.files() {
        let url = app_url.join(file.path())?;
        let bytes = download_exact_bytes(&url, file.size()).await?;

        let actual_hash = bytes_sha256_hex(&bytes);
        if actual_hash != file.sha256() {
            return Err(anyhow!(
                "hash mismatch for {}: expected {}, got {}",
                file.path(),
                file.sha256(),
                actual_hash
            ));
        }

        let output_path = snapshot_dir.join(PathBuf::from(file.path()));
        write_file(&output_path, &bytes)?;
    }

    SageAppSnapshot::new(
        manifest_hash.to_string(),
        snapshot_dir.to_string_lossy().to_string(),
        manifest.clone(),
    )
}

pub(crate) fn write_snapshot_manifest(snapshot: &SageAppSnapshot) -> anyhow::Result<()> {
    let manifest_path = Path::new(snapshot.snapshot_dir()).join(crate::types::MANIFEST_FILE_NAME);

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(snapshot.manifest())?,
    )
    .with_context(|| {
        format!(
            "failed to write snapshot manifest {}",
            manifest_path.display()
        )
    })?;

    Ok(())
}

pub(crate) fn fresh_snapshot_dir(app_dir: &Path) -> PathBuf {
    app_dir
        .join("snapshots")
        .join(uuid::Uuid::new_v4().to_string())
}
