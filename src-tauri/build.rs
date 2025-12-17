use glob::glob;
use std::env;
use std::path::PathBuf;

/// Finds the Android NDK path from environment variables or common locations
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

/// Sets up Android NDK include paths for bindgen
fn setup_android_bindgen() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").ok();
    if target_os.as_deref() != Some("android") {
        return;
    }

    let Some(ndk_path) = find_android_ndk() else {
        eprintln!(
            "Warning: Could not find Android NDK. Set ANDROID_NDK_HOME environment variable."
        );
        return;
    };

    let build_os = match env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => {
            eprintln!("Warning: Unsupported build OS for Android NDK detection.");
            return;
        }
    };

    // Try to find the sysroot include directory
    let arch = env::consts::ARCH;
    let prebuilt_dirs = if arch == "aarch64" || arch == "arm64" {
        vec![
            format!("{}-aarch64", build_os),
            format!("{}-x86_64", build_os),
        ]
    } else {
        vec![format!("{}-x86_64", build_os)]
    };

    for prebuilt_dir in &prebuilt_dirs {
        let sysroot_include = ndk_path
            .join("toolchains/llvm/prebuilt")
            .join(prebuilt_dir)
            .join("sysroot/usr/include");

        if sysroot_include.exists() {
            let sysroot = sysroot_include
                .parent()
                .expect("sysroot_include should always have a parent directory")
                .to_string_lossy();
            let include_path = sysroot_include.to_string_lossy();

            // Get the target architecture for Android
            let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
            let android_arch = match target_arch.as_str() {
                "aarch64" => "aarch64-linux-android",
                "arm" => "arm-linux-androideabi",
                "x86_64" => "x86_64-linux-android",
                "x86" => "i686-linux-android",
                _ => "",
            };

            // Build include paths
            let mut include_args = vec![
                format!("--sysroot={}", sysroot),
                format!("-I{}", include_path),
            ];

            // Add arch-specific include directory if it exists
            if !android_arch.is_empty() {
                let arch_include = sysroot_include
                    .parent()
                    .expect("sysroot_include should always have a parent directory")
                    .join("usr/include")
                    .join(android_arch);
                if arch_include.exists() {
                    include_args.push(format!("-I{}", arch_include.to_string_lossy()));
                }
            }

            // Add C++ include directory if it exists
            let cpp_include = sysroot_include.join("c++").join("v1");
            if cpp_include.exists() {
                include_args.push(format!("-I{}", cpp_include.to_string_lossy()));
            }

            let new_args = include_args.join(" ");
            // Set it in the current process environment (for this build script and its children)
            env::set_var("BINDGEN_EXTRA_CLANG_ARGS", &new_args);
            // Also set it via cargo:rustc-env for subsequent build steps
            println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS={new_args}");
            println!("cargo:warning=Setting BINDGEN_EXTRA_CLANG_ARGS for Android NDK: {new_args}");
            println!("cargo:warning=If bindgen still fails, export BINDGEN_EXTRA_CLANG_ARGS='{new_args}' before running cargo build");
            return;
        }
    }

    eprintln!("Warning: Could not find Android NDK sysroot include directory.");
}

/// Adds a temporary workaround for an issue with the Rust compiler and Android
/// in `x86_64` devices: <https://github.com/rust-lang/rust/issues/109717>.
/// The workaround comes from: <https://github.com/smartvaults/smartvaults/blob/827805a989561b78c0ea5b41f2c1c9e9e59545e0/bindings/smartvaults-sdk-ffi/build.rs>
fn setup_x86_64_android_workaround() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    if target_arch == "x86_64" && target_os == "android" {
        let android_ndk_home =
            find_android_ndk().expect("ANDROID_NDK_HOME not set and could not find NDK");
        let build_os = match env::consts::OS {
            "linux" => "linux",
            "macos" => "darwin",
            "windows" => "windows",
            _ => panic!(
                "Unsupported OS. You must use either Linux, MacOS or Windows to build the crate."
            ),
        };
        let linux_x86_64_lib_pattern = format!(
            "{}/toolchains/llvm/prebuilt/{}-x86_64/lib*/clang/**/lib/linux/",
            android_ndk_home.to_string_lossy(),
            build_os
        );
        match glob(&linux_x86_64_lib_pattern).expect("glob failed").last() {
            Some(Ok(path)) => {
                println!("cargo:rustc-link-search={}", path.to_string_lossy());
                println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");    
            },
            _ => panic!("Path not found: {linux_x86_64_lib_pattern}. Try setting a different ANDROID_NDK_HOME."),
        }
    }
}

fn main() {
    setup_android_bindgen();
    setup_x86_64_android_workaround();
    tauri_build::build();
}
