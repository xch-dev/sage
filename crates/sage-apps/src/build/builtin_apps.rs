use std::path::{Path, PathBuf};

const SAGE_BUILTIN_APPS_DIST_ENV: &str = "SAGE_BUILTIN_APPS_DIST";

pub fn build_builtin_apps() -> Result<(), String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let workspace_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| "crates/sage-apps should have workspace root above it".to_string())?;

    let dist_root = workspace_root
        .join("builtin-apps")
        .join("build")
        .join("dist");

    println!(
        "cargo:rustc-env={SAGE_BUILTIN_APPS_DIST_ENV}={}",
        dist_root.display()
    );

    println!("cargo:rerun-if-changed={}", dist_root.display());

    require_dir(&dist_root)?;
    require_dir(&dist_root.join("sandbox-test"))?;
    require_dir(&dist_root.join("runtime"))?;
    require_dir(&dist_root.join("system"))?;

    Ok(())
}

fn require_dir(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return Ok(());
    }

    Err(format!(
        "missing builtin apps build output at {}. Run `pnpm build:builtin-apps` first.",
        path.display()
    ))
}
