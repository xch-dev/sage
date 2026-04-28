use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult, anyhow};
use zip::ZipArchive;

use crate::types::MANIFEST_FILE_NAME;
use crate::types::{SageAppPackageManifest, SageAppSnapshot};
use crate::utils::bytes_sha256_hex;

pub fn unzip_to_dir(zip_path: &Path, out_dir: &Path) -> AnyResult<()> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("failed to open zip {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).context("failed to read zip archive")?;

    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
    }

    fs::create_dir_all(out_dir)?;

    archive
        .extract(out_dir)
        .context("failed to extract zip archive")?;

    Ok(())
}

pub fn detect_package_root(unpack_dir: &Path) -> AnyResult<PathBuf> {
    let direct_manifest = unpack_dir.join(MANIFEST_FILE_NAME);
    if direct_manifest.is_file() {
        return Ok(unpack_dir.to_path_buf());
    }

    let mut dirs = fs::read_dir(unpack_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(fs::FileType::is_dir)
                .map(|_| entry.path())
        })
        .collect::<Vec<_>>();

    dirs.sort();

    for dir in dirs {
        if dir.join(MANIFEST_FILE_NAME).is_file() {
            return Ok(dir);
        }
    }

    anyhow::bail!("could not find {MANIFEST_FILE_NAME}")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> AnyResult<()> {
    fs::create_dir_all(dst)
        .with_context(|| format!("failed to create directory {}", dst.display()))?;

    for entry in
        fs::read_dir(src).with_context(|| format!("failed to read directory {}", src.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if file_type.is_file() {
            fs::copy(&from, &to).with_context(|| {
                format!("failed to copy {} to {}", from.display(), to.display())
            })?;
        }
    }

    Ok(())
}

pub fn compute_dir_size(root: &Path) -> AnyResult<u64> {
    let mut total = 0_u64;

    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to read directory {}", root.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            total = total
                .checked_add(compute_dir_size(&path)?)
                .ok_or_else(|| anyhow!("directory size overflow"))?;
        } else if file_type.is_file() {
            total = total
                .checked_add(entry.metadata()?.len())
                .ok_or_else(|| anyhow!("directory size overflow"))?;
        }
    }

    Ok(total)
}

pub fn prepare_zip_snapshot(
    package_root: &Path,
    app_dir: &Path,
    manifest: &SageAppPackageManifest,
) -> AnyResult<SageAppSnapshot> {
    let snapshot_dir = app_dir.join("active");

    if snapshot_dir.exists() {
        fs::remove_dir_all(&snapshot_dir).with_context(|| {
            format!(
                "failed to remove existing snapshot dir {}",
                snapshot_dir.display()
            )
        })?;
    }

    copy_dir_recursive(package_root, &snapshot_dir).with_context(|| {
        format!(
            "failed to copy unpacked package {} into snapshot {}",
            package_root.display(),
            snapshot_dir.display()
        )
    })?;

    let manifest_hash = bytes_sha256_hex(&serde_json::to_vec(manifest)?);

    SageAppSnapshot::new(
        manifest_hash,
        snapshot_dir.to_string_lossy().to_string(),
        manifest.clone(),
    )
}
