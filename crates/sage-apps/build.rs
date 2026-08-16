use std::{env, path::Path};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let manifest_path = Path::new(&manifest_dir).join("windows-app-manifest.xml");

    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
        manifest_path.display()
    );
}
