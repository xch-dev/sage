use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result as AnyResult, anyhow};
use futures::{StreamExt, TryStreamExt, stream};

use crate::{
    MANIFEST_FILE_NAME, SageAppPackageManifest, SageAppSnapshot, SageAppUrl, bytes_sha256_hex,
    security::{SsrfGuardedClient, get_with_ssrf_guard},
};

const MAX_CONCURRENT_FILE_DOWNLOADS: usize = 8;

pub(crate) async fn download_bytes_with_limit(url: &str, max_bytes: u64) -> AnyResult<Vec<u8>> {
    let response = get_with_ssrf_guard(url)
        .await?
        .error_for_status()
        .with_context(|| format!("request failed for {url}"))?;

    read_response_bytes_with_limit(url, response, max_bytes, |_| {}).await
}

async fn download_exact_bytes_with_client_and_progress(
    client: &SsrfGuardedClient,
    url: &str,
    expected_size: u64,
    on_progress: &mut impl FnMut(u64),
) -> AnyResult<Vec<u8>> {
    let response = client
        .get(url)
        .await?
        .error_for_status()
        .with_context(|| format!("request failed for {url}"))?;

    let bytes = read_response_bytes_with_limit(url, response, expected_size, on_progress).await?;

    ensure_expected_size(url, &bytes, expected_size)?;

    Ok(bytes)
}

async fn read_response_bytes_with_limit(
    url: &str,
    response: reqwest::Response,
    max_bytes: u64,
    mut on_progress: impl FnMut(u64),
) -> AnyResult<Vec<u8>> {
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
        on_progress(received);
    }

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
    download_url_snapshot_with_progress(snapshot_dir, app_url, manifest, manifest_hash, |_| {})
        .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

pub async fn download_url_snapshot_with_progress(
    snapshot_dir: &Path,
    app_url: &SageAppUrl,
    manifest: &SageAppPackageManifest,
    manifest_hash: &str,
    on_progress: impl Fn(SnapshotDownloadProgress) + Send + Sync,
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

    let total_bytes = manifest.total_bytes();
    let downloaded_bytes = AtomicU64::new(0);
    on_progress(SnapshotDownloadProgress {
        downloaded_bytes: 0,
        total_bytes,
    });

    let client = SsrfGuardedClient::default();

    stream::iter(manifest.files().to_vec())
        .map(|file| {
            let client = &client;
            let downloaded_bytes = &downloaded_bytes;
            let on_progress = &on_progress;
            async move {
                let url = app_url.join(file.path())?;
                let mut file_downloaded_bytes = 0u64;
                let mut report_file_progress = |received: u64| {
                    let delta = received.saturating_sub(file_downloaded_bytes);
                    file_downloaded_bytes = received;
                    let total_downloaded =
                        downloaded_bytes.fetch_add(delta, Ordering::Relaxed) + delta;
                    on_progress(SnapshotDownloadProgress {
                        downloaded_bytes: total_downloaded.min(total_bytes),
                        total_bytes,
                    });
                };
                let bytes = download_exact_bytes_with_client_and_progress(
                    client,
                    &url,
                    file.size(),
                    &mut report_file_progress,
                )
                .await?;

                ensure_expected_hash(file.path(), &bytes, file.sha256())?;

                let output_path = snapshot_dir.join(PathBuf::from(file.path()));
                write_file(&output_path, &bytes)?;

                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(MAX_CONCURRENT_FILE_DOWNLOADS)
        .try_collect::<Vec<_>>()
        .await?;

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
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        thread,
        time::Duration,
    };

    use tempfile::tempdir;

    use super::*;
    use crate::{SageAppManifestFile, SageAppPackageManifestParts, SageRequestedPermissions};

    #[tokio::test]
    async fn downloads_snapshot_files_with_bounded_concurrency() {
        const FILE_COUNT: usize = 12;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let active_requests = Arc::new(AtomicUsize::new(0));
        let max_active_requests = Arc::new(AtomicUsize::new(0));
        let server_active = active_requests.clone();
        let server_max_active = max_active_requests.clone();

        let server = thread::spawn(move || {
            let mut workers = Vec::new();

            for _ in 0..FILE_COUNT {
                let (mut stream, _) = listener.accept().unwrap();
                let active = server_active.clone();
                let max_active = server_max_active.clone();

                workers.push(thread::spawn(move || {
                    let mut request = [0_u8; 4096];
                    let bytes_read = stream.read(&mut request).unwrap();
                    assert!(bytes_read > 0);

                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(current, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(75));

                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx",
                        )
                        .unwrap();
                    active.fetch_sub(1, Ordering::SeqCst);
                }));
            }

            for worker in workers {
                worker.join().unwrap();
            }
        });

        let files = (0..FILE_COUNT)
            .map(|index| {
                SageAppManifestFile::new(format!("file-{index}.txt"), bytes_sha256_hex(b"x"), 1)
                    .unwrap()
            })
            .collect();
        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Concurrent Download Test".to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files,
            entry: Some("file-0.txt".to_string()),
            author: None,
            donation: None,
        })
        .unwrap();
        let app_url = SageAppUrl::parse(format!("http://{address}/")).unwrap();
        let dir = tempdir().unwrap();

        download_url_snapshot(dir.path(), &app_url, &manifest, "manifest-hash")
            .await
            .unwrap();
        server.join().unwrap();

        let observed_max = max_active_requests.load(Ordering::SeqCst);
        assert!(observed_max > 1, "downloads were sequential");
        assert!(observed_max <= MAX_CONCURRENT_FILE_DOWNLOADS);

        for index in 0..FILE_COUNT {
            assert_eq!(
                fs::read(dir.path().join(format!("file-{index}.txt"))).unwrap(),
                b"x"
            );
        }
    }

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
