use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

static BUILTIN_APPS_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before UNIX_EPOCH")
        .as_millis() as i64
}

pub fn bytes_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn slugify_app_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    let out = out.trim_matches('-').to_string();

    if out.is_empty() {
        "app".to_string()
    } else {
        out
    }
}

pub fn set_builtin_apps_root(path: impl Into<PathBuf>) {
    let path = path.into();

    if BUILTIN_APPS_ROOT.set(path.clone()).is_err() {
        tracing::warn!(
            "builtin apps root was already initialized; keeping {}",
            builtin_apps_root().display()
        );
    }
}

pub fn builtin_apps_root() -> PathBuf {
    if let Some(path) = option_env!("SAGE_BUILTIN_APPS_DIST") {
        return PathBuf::from(path);
    }

    if let Some(path) = BUILTIN_APPS_ROOT.get() {
        return path.clone();
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crates/sage-apps should have workspace root above it")
        .join("builtin-apps")
        .join("build")
        .join("dist")
}
