use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult};
use zip::ZipArchive;

use crate::types::MANIFEST_FILE_NAME;
use crate::types::{SageAppPackageManifest, SageAppSnapshot};
use crate::utils::bytes_sha256_hex;

const MAX_ZIP_ENTRIES: usize = 2_000;
const MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES: u64 = 200 * 1024 * 1024;
const MAX_ZIP_SINGLE_FILE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_ZIP_COMPRESSION_RATIO: u64 = 100;

pub fn unzip_to_dir(zip_path: &Path, out_dir: &Path) -> AnyResult<()> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("failed to open zip {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file).context("failed to read zip archive")?;

    if archive.len() > MAX_ZIP_ENTRIES {
        anyhow::bail!("zip archive contains too many entries");
    }

    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
    }

    fs::create_dir_all(out_dir)?;

    let root = out_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize output dir {}", out_dir.display()))?;

    let mut total_uncompressed = 0u64;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("failed to read zip entry #{index}"))?;

        let Some(enclosed_name) = entry.enclosed_name() else {
            anyhow::bail!("zip entry has unsafe path: {}", entry.name());
        };

        let uncompressed_size = entry.size();
        let compressed_size = entry.compressed_size();

        if uncompressed_size > MAX_ZIP_SINGLE_FILE_BYTES {
            anyhow::bail!("zip entry is too large: {}", entry.name());
        }

        if compressed_size > 0 && uncompressed_size / compressed_size > MAX_ZIP_COMPRESSION_RATIO {
            anyhow::bail!("zip entry compression ratio is too high: {}", entry.name());
        }

        total_uncompressed = total_uncompressed
            .checked_add(uncompressed_size)
            .context("zip uncompressed size overflow")?;

        if total_uncompressed > MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES {
            anyhow::bail!("zip archive exceeds maximum extracted size");
        }

        let target = root.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create directory {}", target.display()))?;
            continue;
        }

        let parent = target.parent().context("zip target path has no parent")?;

        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;

        let canonical_parent = parent.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize zip target parent {}",
                parent.display()
            )
        })?;

        if !canonical_parent.starts_with(&root) {
            anyhow::bail!("zip entry escapes extraction directory: {}", entry.name());
        }

        let mut out = fs::File::create(&target)
            .with_context(|| format!("failed to create file {}", target.display()))?;

        io::copy(&mut entry, &mut out)
            .with_context(|| format!("failed to extract zip entry {}", entry.name()))?;
    }

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

pub fn prepare_zip_snapshot(
    package_root: &Path,
    snapshot_dir: &Path,
    manifest: &SageAppPackageManifest,
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

    copy_dir_recursive(package_root, snapshot_dir).with_context(|| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        for (name, contents) in entries {
            zip.start_file(name, options).unwrap();
            zip.write_all(contents).unwrap();
        }

        zip.finish().unwrap();
    }

    #[test]
    fn unzip_to_dir_extracts_normal_nested_files() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("app.zip");
        let out_dir = dir.path().join("out");

        write_zip(
            &zip_path,
            &[
                ("sage-app.json", br#"{"name":"test"}"#),
                ("nested/index.html", b"<html></html>"),
            ],
        );

        unzip_to_dir(&zip_path, &out_dir).unwrap();

        assert_eq!(
            fs::read_to_string(out_dir.join("nested/index.html")).unwrap(),
            "<html></html>"
        );
    }

    #[test]
    fn unzip_to_dir_rejects_parent_directory_escape() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("evil.zip");
        let out_dir = dir.path().join("out");

        write_zip(&zip_path, &[("../evil.txt", b"owned")]);

        let err = unzip_to_dir(&zip_path, &out_dir)
            .expect_err("zip entries escaping output dir must be rejected");

        let message = err.to_string();
        assert!(
            message.contains("unsafe path") || message.contains("escapes extraction directory"),
            "unexpected error: {message}"
        );

        assert!(!dir.path().join("evil.txt").exists());
    }

    #[test]
    fn unzip_to_dir_keeps_absolute_path_inside_output_dir() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("absolute.zip");
        let out_dir = dir.path().join("out");

        write_zip(&zip_path, &[("/tmp/sage-apps-test-file.txt", b"owned")]);

        unzip_to_dir(&zip_path, &out_dir).unwrap();

        assert!(out_dir.join("tmp/sage-apps-test-file.txt").is_file());
        assert_eq!(
            fs::read_to_string(out_dir.join("tmp/sage-apps-test-file.txt")).unwrap(),
            "owned"
        );
    }

    #[test]
    fn unzip_to_dir_rejects_too_large_single_file() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("large.zip");
        let out_dir = dir.path().join("out");

        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("large.bin", SimpleFileOptions::default())
            .unwrap();

        let chunk = vec![0u8; 1024 * 1024];
        for _ in 0..=MAX_ZIP_SINGLE_FILE_BYTES / chunk.len() as u64 {
            zip.write_all(&chunk).unwrap();
        }

        zip.finish().unwrap();

        let err =
            unzip_to_dir(&zip_path, &out_dir).expect_err("oversized zip entry must be rejected");

        assert!(
            err.to_string().contains("too large"),
            "unexpected error: {err}"
        );
    }
}
