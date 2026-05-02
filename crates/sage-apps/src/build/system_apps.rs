use std::{fs, path::Path};

use super::finalize::finalize_prebuilt_app;

pub fn build_system_apps(
    system_apps_src_dir: &Path,
    system_out_dir: &Path,
    system_sdk_dist: &Path,
) -> Result<(), String> {
    let work_system_apps_dir = system_out_dir
        .parent()
        .and_then(|dist_dir| dist_dir.parent())
        .ok_or_else(|| {
            format!(
                "failed to resolve builtin apps root from {}",
                system_out_dir.display()
            )
        })?
        .join("work")
        .join("system-apps");

    let mut app_dirs = fs::read_dir(system_apps_src_dir)
        .map_err(|err| {
            format!(
                "failed to read system apps source directory {}: {err}",
                system_apps_src_dir.display()
            )
        })?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to read system app directory entry: {err}"))?;

    app_dirs.sort();

    for app_src_dir in app_dirs {
        if !app_src_dir.is_dir() {
            continue;
        }

        let app_dir_name = app_src_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "invalid system app directory name at {}",
                    app_src_dir.display()
                )
            })?;

        let manifest_src = app_src_dir.join("sage-manifest.json");
        if !manifest_src.is_file() {
            continue;
        }

        let app_dist_dir = work_system_apps_dir.join(app_dir_name).join("dist");
        let out_dir = system_out_dir.join(app_dir_name);

        if !app_dist_dir.is_dir() {
            return Err(format!(
                "missing built system app dist directory at {}",
                app_dist_dir.display()
            ));
        }

        if out_dir.exists() {
            fs::remove_dir_all(&out_dir)
                .map_err(|err| format!("failed to remove {}: {err}", out_dir.display()))?;
        }

        fs::create_dir_all(&out_dir)
            .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

        copy_dir_contents(&app_dist_dir, &out_dir)?;

        finalize_prebuilt_app(&out_dir, &manifest_src, &out_dir, system_sdk_dist)?;
    }

    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|err| format!("failed to read {}: {err}", src.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to stat {}: {err}", src_path.display()))?;

        if metadata.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|err| format!("failed to create {}: {err}", dst_path.display()))?;
            copy_dir_contents(&src_path, &dst_path)?;
        } else if metadata.is_file() {
            fs::copy(&src_path, &dst_path).map_err(|err| {
                format!(
                    "failed to copy {} to {}: {err}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }

    Ok(())
}
