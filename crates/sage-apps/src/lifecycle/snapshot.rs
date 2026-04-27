use std::{
    fs,
    path::{Path, PathBuf},
};
use std::path::Component;
use anyhow::{Context, Result as AnyResult, anyhow};
use crate::lifecycle::compute_dir_size;
use crate::types::{SageAppPackageManifest, SageAppSnapshot};
use crate::utils::bytes_sha256_hex;

pub fn read_snapshot_file(root: &Path, request_path: &str) -> AnyResult<PathBuf> {
    let normalized = if request_path.is_empty() || request_path == "/" {
        "index.html"
    } else {
        request_path.trim_start_matches('/')
    };

    let relative = Path::new(normalized);

    if relative.is_absolute() {
        return Err(anyhow!("snapshot path must be relative"));
    }

    for component in relative.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(anyhow!(
                    "invalid snapshot path component in {}",
                    request_path
                ));
            }
        }
    }

    let path = root.join(relative);

    if !path.is_file() {
        return Err(anyhow!("snapshot file not found: {}", request_path));
    }

    Ok(path)
}

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

    fs::write(path, bytes)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

fn join_app_url(base_url: &str, relative_path: &str) -> AnyResult<String> {
    let base = reqwest::Url::parse(base_url)
        .with_context(|| format!("invalid app url {base_url}"))?;
    let joined = base
        .join(relative_path)
        .with_context(|| format!("failed to join app url {base_url} with path {relative_path}"))?;
    Ok(joined.to_string())
}

pub async fn download_url_snapshot(
    app_dir: &Path,
    app_url: &str,
    manifest: &SageAppPackageManifest,
    manifest_hash: &str,
) -> AnyResult<SageAppSnapshot> {
    let snapshot_dir = app_dir.join("active");

    if snapshot_dir.exists() {
        fs::remove_dir_all(&snapshot_dir).with_context(|| {
            format!("failed to remove existing snapshot dir {}", snapshot_dir.display())
        })?;
    }

    fs::create_dir_all(&snapshot_dir)
        .with_context(|| format!("failed to create snapshot dir {}", snapshot_dir.display()))?;

    for file in manifest.files() {
        let url = join_app_url(app_url, &file.path)?;
        let bytes = download_bytes(&url).await?;

        let actual_hash = bytes_sha256_hex(&bytes);
        if actual_hash != file.sha256 {
            return Err(anyhow!(
                "hash mismatch for {}: expected {}, got {}",
                file.path,
                file.sha256,
                actual_hash
            ));
        }

        let output_path = snapshot_dir.join(PathBuf::from(&file.path));
        write_file(&output_path, &bytes)?;
    }

    let total_bytes = compute_dir_size(&snapshot_dir)?;

    Ok(SageAppSnapshot {
        manifest_hash: manifest_hash.to_string(),
        snapshot_dir: snapshot_dir.to_string_lossy().to_string(),
        total_bytes,
        manifest: manifest.clone(),
    })
}
