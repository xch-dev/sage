use glob::glob;
use std::env;
use std::path::PathBuf;

/// Finds the Android NDK path from environment variables or common locations.
/// Used by the x86_64 Android workaround.
fn find_android_ndk() -> Option<PathBuf> {
    // Try environment variables first
    if let Ok(ndk_home) = env::var("ANDROID_NDK_HOME") {
        return Some(PathBuf::from(ndk_home));
    }
    if let Ok(ndk) = env::var("ANDROID_NDK") {
        return Some(PathBuf::from(ndk));
    }

    // Try common Android SDK locations
    let home = env::var("HOME").ok()?;
    let sdk_paths = [
        format!("{home}/Library/Android/sdk/ndk"),
        format!("{home}/.android/ndk"),
        format!("{home}/Android/Sdk/ndk"),
    ];

    for sdk_path in &sdk_paths {
        let path = PathBuf::from(sdk_path);
        if path.exists() {
            // Find the latest NDK version
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut versions: Vec<_> = entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                versions.sort();
                if let Some(ndk_path) = versions.last() {
                    return Some(ndk_path.clone());
                }
            }
        }
    }

    None
}

/// Adds a temporary workaround for an issue with the Rust compiler and Android
/// on x86_64 devices: <https://github.com/rust-lang/rust/issues/109717>.
/// The workaround comes from: <https://github.com/nicholascz/cargo-ndk/issues/22>
///
/// This is needed until the Rust compiler fixes the upstream issue.
fn setup_x86_64_android_workaround() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch != "x86_64" || target_os != "android" {
        return;
    }

    let Some(android_ndk_home) = find_android_ndk() else {
        eprintln!(
            "Warning: Could not find Android NDK for x86_64 workaround. Set ANDROID_NDK_HOME."
        );
        return;
    };

    let build_os = match env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => {
            eprintln!("Warning: Unsupported build OS for x86_64 Android workaround.");
            return;
        }
    };

    let lib_pattern = format!(
        "{}/toolchains/llvm/prebuilt/{}-x86_64/lib*/clang/**/lib/linux/",
        android_ndk_home.to_string_lossy(),
        build_os
    );

    match glob(&lib_pattern).expect("glob pattern failed").last() {
        Some(Ok(path)) => {
            println!("cargo:rustc-link-search={}", path.to_string_lossy());
            println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
        }
        _ => {
            eprintln!(
                "Warning: Could not find clang_rt.builtins for x86_64 Android: {lib_pattern}"
            );
        }
    }
}

fn main() {
    setup_x86_64_android_workaround();
    tauri_build::build();
}
