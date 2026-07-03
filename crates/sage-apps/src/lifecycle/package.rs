use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{MANIFEST_FILE_NAME, SageAppPackageManifest, SageAppSnapshot, bytes_sha256_hex};

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

    // Extract into a private staging directory and only swap it into place once
    // extraction fully succeeds. This means a rejected or partially-extracted
    // archive (e.g. a zip bomb caught mid-stream) never leaves a usable tree at
    // `out_dir`, and nothing can observe `out_dir` while it is half-written.
    let parent = out_dir
        .parent()
        .context("output dir has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    let staging = staging_dir_for(out_dir);
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)
        .with_context(|| format!("failed to create staging dir {}", staging.display()))?;

    if let Err(err) = extract_archive_into(&mut archive, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(err);
    }

    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
    }

    if let Err(err) = fs::rename(&staging, out_dir) {
        let _ = fs::remove_dir_all(&staging);
        return Err(err).with_context(|| {
            format!("failed to move extracted package into {}", out_dir.display())
        });
    }

    Ok(())
}

fn staging_dir_for(out_dir: &Path) -> PathBuf {
    let file_name = out_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "package".to_string());
    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));

    parent.join(format!(".{file_name}.staging-{}", Uuid::new_v4()))
}

fn extract_archive_into(archive: &mut ZipArchive<fs::File>, staging: &Path) -> AnyResult<()> {
    let root = staging
        .canonicalize()
        .with_context(|| format!("failed to canonicalize staging dir {}", staging.display()))?;

    let mut total_written = 0u64;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("failed to read zip entry #{index}"))?;

        let Some(enclosed_name) = entry.enclosed_name() else {
            anyhow::bail!("zip entry has unsafe path: {}", entry.name());
        };

        let uncompressed_size = entry.size();
        let compressed_size = entry.compressed_size();

        // Cheap up-front rejection using the header-declared sizes. These are
        // attacker-controlled, so they are only a fast path; the authoritative
        // limits below are enforced against the actual decompressed byte count.
        if uncompressed_size > MAX_ZIP_SINGLE_FILE_BYTES {
            anyhow::bail!("zip entry is too large: {}", entry.name());
        }

        if compressed_size > 0 && uncompressed_size / compressed_size > MAX_ZIP_COMPRESSION_RATIO {
            anyhow::bail!("zip entry compression ratio is too high: {}", entry.name());
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

        // Enforce the size caps on the *actual* decompressed stream rather than
        // the header metadata, so a zip whose header understates its real size
        // cannot bomb the disk. Read at most one byte past the allowed limit so
        // we can detect (and reject) an entry that would exceed it.
        let remaining_total = MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES.saturating_sub(total_written);
        let per_entry_limit = remaining_total.min(MAX_ZIP_SINGLE_FILE_BYTES);

        let mut limited = (&mut entry).take(per_entry_limit + 1);
        let written = io::copy(&mut limited, &mut out)
            .with_context(|| format!("failed to extract zip entry {}", entry.name()))?;

        if written > MAX_ZIP_SINGLE_FILE_BYTES {
            anyhow::bail!("zip entry is too large: {}", entry.name());
        }

        total_written = total_written
            .checked_add(written)
            .context("zip uncompressed size overflow")?;

        if total_written > MAX_ZIP_TOTAL_UNCOMPRESSED_BYTES {
            anyhow::bail!("zip archive exceeds maximum extracted size");
        }
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
    validate_package_has_no_undeclared_files(package_root, manifest)?;

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

fn validate_package_has_no_undeclared_files(
    package_root: &Path,
    manifest: &SageAppPackageManifest,
) -> AnyResult<()> {
    let declared = manifest
        .files()
        .iter()
        .map(|file| file.path().to_string())
        .collect::<BTreeSet<_>>();

    validate_dir_has_no_undeclared_files(package_root, package_root, &declared)
}

fn validate_dir_has_no_undeclared_files(
    package_root: &Path,
    dir: &Path,
    declared: &BTreeSet<String>,
) -> AnyResult<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            validate_dir_has_no_undeclared_files(package_root, &path, declared)?;
        } else if file_type.is_file() {
            let relative_path = package_relative_path(package_root, &path)?;

            if relative_path != MANIFEST_FILE_NAME && !declared.contains(&relative_path) {
                anyhow::bail!("package contains undeclared file: {relative_path}");
            }
        } else {
            anyhow::bail!("package contains unsupported file type: {}", path.display());
        }
    }

    Ok(())
}

fn package_relative_path(package_root: &Path, path: &Path) -> AnyResult<String> {
    let relative = path.strip_prefix(package_root).with_context(|| {
        format!(
            "failed to compute package relative path for {}",
            path.display()
        )
    })?;

    let mut parts = Vec::new();

    for component in relative.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str().with_context(|| {
                    format!("package path is not valid UTF-8: {}", path.display())
                })?;

                parts.push(part);
            }
            _ => anyhow::bail!("package path has invalid component: {}", path.display()),
        }
    }

    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    use super::*;
    use crate::{
        SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions,
    };

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

    fn sample_manifest_file(path: &str, bytes: &[u8]) -> SageAppManifestFile {
        SageAppManifestFile::new(path, bytes_sha256_hex(bytes), bytes.len() as u64).unwrap()
    }

    fn sample_manifest(files: Vec<SageAppManifestFile>) -> SageAppPackageManifest {
        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::new(
                SageRequestedNetworkPermissions::empty(),
                SageRequestedCapabilities::empty(),
            )
            .unwrap(),
            files,
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
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

    fn staging_dir_count(parent: &Path) -> usize {
        fs::read_dir(parent)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(".out.staging-")
            })
            .count()
    }

    #[test]
    fn unzip_to_dir_leaves_no_staging_dir_on_success() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("app.zip");
        let out_dir = dir.path().join("out");

        write_zip(&zip_path, &[("index.html", b"<html></html>")]);

        unzip_to_dir(&zip_path, &out_dir).unwrap();

        assert!(out_dir.join("index.html").is_file());
        assert_eq!(staging_dir_count(dir.path()), 0);
    }

    #[test]
    fn unzip_to_dir_leaves_no_staging_dir_on_rejection() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("evil.zip");
        let out_dir = dir.path().join("out");

        write_zip(&zip_path, &[("../evil.txt", b"owned")]);

        unzip_to_dir(&zip_path, &out_dir).unwrap_err();

        assert!(!out_dir.exists());
        assert_eq!(staging_dir_count(dir.path()), 0);
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

    #[test]
    fn prepare_zip_snapshot_rejects_undeclared_files() {
        let dir = tempdir().unwrap();
        let package_root = dir.path().join("package");
        let snapshot_dir = dir.path().join("snapshot");
        fs::create_dir_all(&package_root).unwrap();
        fs::write(package_root.join("index.html"), b"<html></html>").unwrap();
        fs::write(package_root.join("extra.js"), b"alert(1)").unwrap();

        let manifest = sample_manifest(vec![sample_manifest_file("index.html", b"<html></html>")]);

        let err = prepare_zip_snapshot(&package_root, &snapshot_dir, &manifest)
            .expect_err("undeclared files must be rejected");

        assert!(
            err.to_string().contains("undeclared file: extra.js"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn prepare_zip_snapshot_rejects_nested_undeclared_files() {
        let dir = tempdir().unwrap();
        let package_root = dir.path().join("package");
        let snapshot_dir = dir.path().join("snapshot");
        fs::create_dir_all(package_root.join("assets")).unwrap();
        fs::write(package_root.join("index.html"), b"<html></html>").unwrap();
        fs::write(package_root.join("assets/extra.js"), b"alert(1)").unwrap();

        let manifest = sample_manifest(vec![sample_manifest_file("index.html", b"<html></html>")]);

        let err = prepare_zip_snapshot(&package_root, &snapshot_dir, &manifest)
            .expect_err("nested undeclared files must be rejected");

        assert!(
            err.to_string().contains("undeclared file: assets/extra.js"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn prepare_zip_snapshot_accepts_declared_nested_files() {
        let dir = tempdir().unwrap();
        let package_root = dir.path().join("package");
        let snapshot_dir = dir.path().join("snapshot");
        fs::create_dir_all(package_root.join("assets")).unwrap();
        fs::write(package_root.join("index.html"), b"<html></html>").unwrap();
        fs::write(package_root.join("assets/app.js"), b"console.log('ok')").unwrap();

        let manifest = sample_manifest(vec![
            sample_manifest_file("index.html", b"<html></html>"),
            sample_manifest_file("assets/app.js", b"console.log('ok')"),
        ]);

        prepare_zip_snapshot(&package_root, &snapshot_dir, &manifest)
            .expect("declared nested files should be accepted");

        assert!(snapshot_dir.join("assets/app.js").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn prepare_zip_snapshot_rejects_symlinks() {
        let dir = tempdir().unwrap();
        let package_root = dir.path().join("package");
        let snapshot_dir = dir.path().join("snapshot");
        fs::create_dir_all(&package_root).unwrap();
        fs::write(package_root.join("index.html"), b"<html></html>").unwrap();
        std::os::unix::fs::symlink("index.html", package_root.join("index-link.html")).unwrap();

        let manifest = sample_manifest(vec![sample_manifest_file("index.html", b"<html></html>")]);

        let err = prepare_zip_snapshot(&package_root, &snapshot_dir, &manifest)
            .expect_err("symlinks must be rejected");

        assert!(
            err.to_string().contains("unsupported file type"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn prepare_zip_snapshot_allows_manifest_file() {
        let dir = tempdir().unwrap();
        let package_root = dir.path().join("package");
        let snapshot_dir = dir.path().join("snapshot");
        fs::create_dir_all(&package_root).unwrap();
        fs::write(package_root.join("index.html"), b"<html></html>").unwrap();

        let manifest = sample_manifest(vec![sample_manifest_file("index.html", b"<html></html>")]);
        fs::write(
            package_root.join(MANIFEST_FILE_NAME),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        prepare_zip_snapshot(&package_root, &snapshot_dir, &manifest)
            .expect("declared files plus manifest should be accepted");

        assert!(snapshot_dir.join("index.html").is_file());
        assert!(snapshot_dir.join(MANIFEST_FILE_NAME).is_file());
    }
}
