use crate::build::{runtime_apps, system_apps, test_apps};
use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_builtin_apps() -> Result<(), String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let builtin_root = manifest_dir.join("builtin-apps");

    let test_src_dir = builtin_root.join("test-apps-src");
    let runtime_src_dir = builtin_root.join("runtime-apps-src");
    let system_apps_workspace_dir = builtin_root.join("system-apps-src");
    let system_apps_src_dir = system_apps_workspace_dir.join("apps");

    let dist_root = builtin_root.join("dist");
    let test_out_dir = dist_root.join("test-apps");
    let runtime_out_dir = dist_root.join("runtime-apps");
    let system_out_dir = dist_root.join("system-apps");

    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "crates/sage-apps should have workspace root above it".to_string())?;

    let user_sdk_dist = workspace_root
        .join("packages")
        .join("sage-app-sdk")
        .join("dist");

    let system_sdk_dist = workspace_root
        .join("packages")
        .join("sage-system-app-sdk")
        .join("dist");

    println!("cargo:rerun-if-changed={}", test_src_dir.display());
    println!("cargo:rerun-if-changed={}", runtime_src_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        system_apps_workspace_dir.display()
    );
    println!("cargo:rerun-if-changed={}", user_sdk_dist.display());
    println!("cargo:rerun-if-changed={}", system_sdk_dist.display());

    fs::create_dir_all(&test_out_dir)
        .map_err(|err| format!("failed to create {}: {err}", test_out_dir.display()))?;
    fs::create_dir_all(&runtime_out_dir)
        .map_err(|err| format!("failed to create {}: {err}", runtime_out_dir.display()))?;
    fs::create_dir_all(&system_out_dir)
        .map_err(|err| format!("failed to create {}: {err}", system_out_dir.display()))?;

    test_apps::build_test_apps(&test_src_dir, &test_out_dir, &user_sdk_dist)?;
    runtime_apps::build_runtime_apps(&runtime_src_dir, &runtime_out_dir, &user_sdk_dist)?;
    system_apps::build_system_apps(&system_apps_src_dir, &system_out_dir, &system_sdk_dist)?;

    inject_all_manifests(&test_out_dir)?;
    inject_all_manifests(&runtime_out_dir)?;
    inject_all_manifests(&system_out_dir)?;

    Ok(())
}

fn inject_all_manifests(root: &Path) -> Result<(), String> {
    for entry in
        fs::read_dir(root).map_err(|err| format!("failed to read {}: {err}", root.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let app_dir = entry.path();

        if app_dir.is_dir() && app_dir.join("sage-manifest.json").is_file() {
            inject_manifest_files(&app_dir)?;
        }
    }

    Ok(())
}

fn inject_manifest_files(app_dir: &Path) -> Result<(), String> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|err| format!("failed to read {}: {err}", manifest_path.display()))?;

    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_text)
        .map_err(|err| format!("failed to parse {}: {err}", manifest_path.display()))?;

    let mut files = Vec::new();
    collect_manifest_files(app_dir, app_dir, &mut files)?;

    files.sort_by(|a, b| {
        a["path"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["path"].as_str().unwrap_or_default())
    });

    manifest["files"] = serde_json::Value::Array(files);

    let next_text = serde_json::to_string_pretty(&manifest)
        .map_err(|err| format!("failed to serialize {}: {err}", manifest_path.display()))?;
    let next_text = format!("{next_text}\n");

    if manifest_text == next_text {
        return Ok(());
    }

    fs::write(&manifest_path, next_text)
        .map_err(|err| format!("failed to write {}: {err}", manifest_path.display()))?;

    Ok(())
}

fn collect_manifest_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|err| format!("failed to read {}: {err}", dir.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;

        if metadata.is_dir() {
            collect_manifest_files(root, &path, files)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .map_err(|err| format!("failed to relativize {}: {err}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");

        if rel == "sage-manifest.json" {
            continue;
        }

        let bytes =
            fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;

        let sha256 = hex::encode(sha2::Sha256::digest(&bytes));

        files.push(serde_json::json!({
            "path": rel,
            "sha256": sha256,
            "size": metadata.len(),
        }));
    }

    Ok(())
}
