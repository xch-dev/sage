use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult, anyhow};

use crate::{
    MANIFEST_FILE_NAME, SageAppPackageManifest, SageAppSnapshot, SageAppUrl, bytes_sha256_hex,
    security::get_with_ssrf_guard,
};

pub(crate) async fn download_bytes_with_limit(url: &str, max_bytes: u64) -> AnyResult<Vec<u8>> {
    let response = get_with_ssrf_guard(url)
        .await?
        .error_for_status()
        .with_context(|| format!("request failed for {url}"))?;

    if let Some(content_length) = response.content_length() {
        ensure_within_max_response_size(url, content_length, max_bytes)?;
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

        ensure_within_max_response_size(url, received, max_bytes)?;

        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
}

async fn download_exact_bytes(url: &str, expected_size: u64) -> AnyResult<Vec<u8>> {
    let bytes = download_bytes_with_limit(url, expected_size).await?;

    ensure_expected_size(url, &bytes, expected_size)?;

    Ok(bytes)
}

fn ensure_within_max_response_size(url: &str, received: u64, max_bytes: u64) -> AnyResult<()> {
    if received > max_bytes {
        anyhow::bail!("response body from {url} exceeds maximum size {max_bytes}");
    }

    Ok(())
}

fn ensure_expected_size(url: &str, bytes: &[u8], expected_size: u64) -> AnyResult<()> {
    if u64::try_from(bytes.len()).context("response body too large")? != expected_size {
        anyhow::bail!("response body from {url} did not match expected size {expected_size}");
    }

    Ok(())
}

fn ensure_expected_hash(path: &str, bytes: &[u8], expected_hash: &str) -> AnyResult<()> {
    let actual_hash = bytes_sha256_hex(bytes);
    if actual_hash != expected_hash {
        return Err(anyhow!(
            "hash mismatch for {path}: expected {expected_hash}, got {actual_hash}"
        ));
    }

    Ok(())
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

        ensure_expected_hash(file.path(), &bytes, file.sha256())?;

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
    let manifest_path = Path::new(snapshot.snapshot_dir()).join(MANIFEST_FILE_NAME);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_expected_size_accepts_exact_size() {
        ensure_expected_size("https://example.com/index.html", b"hello", 5).unwrap();
    }

    #[test]
    fn ensure_within_max_response_size_accepts_limit() {
        ensure_within_max_response_size("https://example.com/index.html", 5, 5).unwrap();
    }

    #[test]
    fn ensure_within_max_response_size_rejects_over_limit() {
        let err = ensure_within_max_response_size("https://example.com/index.html", 6, 5)
            .expect_err("over-limit body must be rejected");

        assert!(
            err.to_string().contains("exceeds maximum size 5"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ensure_expected_size_rejects_larger_body() {
        let err = ensure_expected_size("https://example.com/index.html", b"too-large", 3)
            .expect_err("oversized body must be rejected");
        assert!(
            err.to_string().contains("did not match expected size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ensure_expected_size_rejects_smaller_body() {
        let err = ensure_expected_size("https://example.com/index.html", b"tiny", 6)
            .expect_err("undersized body must be rejected");
        assert!(
            err.to_string().contains("did not match expected size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ensure_expected_hash_accepts_matching_hash() {
        let bytes = b"hello";
        let hash = bytes_sha256_hex(bytes);
        ensure_expected_hash("index.html", bytes, &hash).unwrap();
    }

    #[test]
    fn ensure_expected_hash_rejects_mismatch() {
        let err = ensure_expected_hash("index.html", b"wrong", &bytes_sha256_hex(b"right"))
            .expect_err("hash mismatch must be rejected");
        assert!(
            err.to_string().contains("hash mismatch"),
            "unexpected error: {err}"
        );
    }
}
