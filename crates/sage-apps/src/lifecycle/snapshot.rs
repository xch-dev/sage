use crate::types::{SageAppPackageManifest, SageAppSnapshot, SageAppUrl};
use crate::utils::bytes_sha256_hex;
use anyhow::{Context, Result as AnyResult, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
};

async fn download_bytes(url: &str) -> AnyResult<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("request failed for {url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read response body from {url}"))?;

    Ok(bytes.to_vec())
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
        let bytes = download_bytes(&url).await?;

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
