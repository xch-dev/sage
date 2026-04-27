use glob::glob;
use std::env;

fn setup_x86_64_android_workaround() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    if target_arch == "x86_64" && target_os == "android" {
        let android_ndk_home = env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME not set");

        let build_os = match env::consts::OS {
            "linux" => "linux",
            "macos" => "darwin",
            "windows" => "windows",
            _ => panic!(
                "Unsupported OS. You must use either Linux, MacOS or Windows to build the crate."
            ),
        };

        let linux_x86_64_lib_pattern = format!(
            "{android_ndk_home}/toolchains/llvm/prebuilt/{build_os}-x86_64/lib*/clang/**/lib/linux/"
        );

        match glob(&linux_x86_64_lib_pattern).expect("glob failed").last() {
            Some(Ok(path)) => {
                println!("cargo:rustc-link-search={}", path.to_string_lossy());
                println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
            }
            _ => panic!(
                "Path not found: {linux_x86_64_lib_pattern}. Try setting a different ANDROID_NDK_HOME."
            ),
        }
    }
}

fn main() {
    setup_x86_64_android_workaround();

    println!("cargo:rerun-if-changed=../crates/sage-apps/src/bridge");
    println!("cargo:rerun-if-changed=../crates/sage-apps/src/permissions");
    println!("cargo:rerun-if-changed=../crates/sage-apps/src/build/docs.rs");
    println!("cargo:rerun-if-changed=../crates/sage-apps/src/build/builtin_apps.rs");
    println!("cargo:rerun-if-changed=../crates/sage-apps/builtin-apps/test-apps-src");
    println!("cargo:rerun-if-changed=../crates/sage-apps/builtin-apps/runtime-apps-src");
    println!("cargo:rerun-if-changed=../crates/sage-apps/builtin-apps/system-apps-src");

    if let Err(err) = sage_apps::build::builtin_apps::build_builtin_apps() {
        panic!("failed to build builtin apps: {err}");
    }

    if let Err(err) = sage_apps::build::docs::generate_docs() {
        panic!("failed to generate Sage app docs: {err}");
    }

    tauri_build::build();
}
