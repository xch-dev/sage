use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn files_equal(src: &Path, dst: &Path) -> Result<bool, String> {
    if !src.is_file() || !dst.is_file() {
        return Ok(false);
    }

    let src_meta =
        fs::metadata(src).map_err(|err| format!("failed to stat {}: {err}", src.display()))?;
    let dst_meta =
        fs::metadata(dst).map_err(|err| format!("failed to stat {}: {err}", dst.display()))?;

    if src_meta.len() != dst_meta.len() {
        return Ok(false);
    }

    let src_bytes =
        fs::read(src).map_err(|err| format!("failed to read {}: {err}", src.display()))?;
    let dst_bytes =
        fs::read(dst).map_err(|err| format!("failed to read {}: {err}", dst.display()))?;

    Ok(src_bytes == dst_bytes)
}

pub fn copy_file_if_changed(src: &Path, dst: &Path, label: &str) -> Result<(), String> {
    if !src.is_file() {
        return Err(format!("missing {label} at {}", src.display()));
    }

    if files_equal(src, dst)? {
        return Ok(());
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }

    fs::copy(src, dst).map_err(|err| {
        format!(
            "failed to copy {} -> {}: {err}",
            src.display(),
            dst.display()
        )
    })?;

    Ok(())
}

pub fn collect_files_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|err| format!("failed to read {}: {err}", current.display()))?
    {
        let entry =
            entry.map_err(|err| format!("failed to read entry in {}: {err}", current.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;

        if file_type.is_dir() {
            collect_files_recursive(root, &path, out)?;
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|err| {
                    format!(
                        "failed to strip prefix {} from {}: {err}",
                        root.display(),
                        path.display()
                    )
                })?
                .to_path_buf();
            out.push(rel);
        }
    }

    Ok(())
}

pub fn copy_dir_all_if_changed(
    src: &Path,
    dst: &Path,
    expected_files: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!("missing directory: {}", src.display()));
    }

    fs::create_dir_all(dst).map_err(|err| format!("failed to create {}: {err}", dst.display()))?;

    let mut rel_files = Vec::new();
    collect_files_recursive(src, src, &mut rel_files)?;

    for rel in rel_files {
        let src_path = src.join(&rel);
        let dst_path = dst.join(&rel);
        copy_file_if_changed(&src_path, &dst_path, "built app file")?;
        expected_files.insert(rel);
    }

    Ok(())
}

pub fn remove_unexpected_files(
    dir: &Path,
    expected_files: &BTreeSet<PathBuf>,
) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut existing_files = Vec::new();
    collect_files_recursive(dir, dir, &mut existing_files)?;

    for rel in existing_files {
        if !expected_files.contains(&rel) {
            let path = dir.join(&rel);
            fs::remove_file(&path)
                .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
        }
    }

    remove_empty_dirs_recursively(dir)?;
    Ok(())
}

pub fn remove_empty_dirs_recursively(dir: &Path) -> Result<bool, String> {
    if !dir.is_dir() {
        return Ok(false);
    }

    let mut is_empty = true;

    for entry in
        fs::read_dir(dir).map_err(|err| format!("failed to read {}: {err}", dir.display()))?
    {
        let entry =
            entry.map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;

        if file_type.is_dir() {
            let child_empty = remove_empty_dirs_recursively(&path)?;
            if child_empty {
                fs::remove_dir(&path)
                    .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
            } else {
                is_empty = false;
            }
        } else {
            is_empty = false;
        }
    }

    Ok(is_empty)
}
