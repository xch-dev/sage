use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::fs_utils::{copy_dir_all_if_changed, copy_file_if_changed, remove_unexpected_files};

pub fn finalize_source_app(
    shared_dir: Option<&Path>,
    source_dir: &Path,
    out_dir: &Path,
    manifest_file_name: Option<&str>,
    sdk_dist: &Path,
) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

    let mut expected_files = BTreeSet::<PathBuf>::new();

    if let Some(shared_dir) = shared_dir
        && shared_dir.is_dir()
    {
        copy_dir_all_if_changed(shared_dir, out_dir, &mut expected_files)?;
    }

    copy_dir_all_if_changed(source_dir, out_dir, &mut expected_files)?;

    if let Some(manifest_file_name) = manifest_file_name {
        let rel = PathBuf::from("sage-manifest.json");
        copy_file_if_changed(
            &source_dir.join(manifest_file_name),
            &out_dir.join(&rel),
            &format!("manifest {manifest_file_name}"),
        )?;
        expected_files.insert(rel);

        for legacy_name in [
            "sage-manifest.json",
            "sage-manifest.persistent.json",
            "sage-manifest.incognito.json",
        ] {
            let rel = PathBuf::from(legacy_name);
            if rel != PathBuf::from("sage-manifest.json") {
                let path = out_dir.join(&rel);
                if path.is_file() {
                    fs::remove_file(&path)
                        .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
                }
            }
        }
    }

    let bridge_rel = PathBuf::from("bridge.js");
    copy_file_if_changed(
        &sdk_dist.join("runtime-bridge.js"),
        &out_dir.join(&bridge_rel),
        "SDK runtime bridge build output",
    )?;
    expected_files.insert(bridge_rel);

    let sdk_rel = PathBuf::from("sdk.js");
    copy_file_if_changed(
        &sdk_dist.join("index.js"),
        &out_dir.join(&sdk_rel),
        "SDK index build output",
    )?;
    expected_files.insert(sdk_rel);

    remove_unexpected_files(out_dir, &expected_files)?;
    Ok(())
}

pub fn finalize_prebuilt_app(
    built_dir: &Path,
    manifest_src: &Path,
    out_dir: &Path,
    sdk_dist: &Path,
) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

    let mut expected_files = BTreeSet::<PathBuf>::new();

    copy_dir_all_if_changed(built_dir, out_dir, &mut expected_files)?;

    let manifest_rel = PathBuf::from("sage-manifest.json");
    copy_file_if_changed(
        manifest_src,
        &out_dir.join(&manifest_rel),
        "system app manifest",
    )?;
    expected_files.insert(manifest_rel);

    let bridge_rel = PathBuf::from("bridge.js");
    copy_file_if_changed(
        &sdk_dist.join("runtime-bridge.js"),
        &out_dir.join(&bridge_rel),
        "system SDK runtime bridge build output",
    )?;
    expected_files.insert(bridge_rel);

    let sdk_rel = PathBuf::from("sdk.js");
    copy_file_if_changed(
        &sdk_dist.join("index.js"),
        &out_dir.join(&sdk_rel),
        "system SDK index build output",
    )?;
    expected_files.insert(sdk_rel);

    remove_unexpected_files(out_dir, &expected_files)?;
    Ok(())
}
