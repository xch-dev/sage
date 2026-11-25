#[cfg(target_os = "windows")]
use std::path::{Path, PathBuf};

/// The WebView2 user data folder of a Windows profile. It must be absolute, because WebView2
/// resolves relative paths against the process working directory.
#[cfg(target_os = "windows")]
pub fn windows_profiles_dir(webview_root: &Path) -> PathBuf {
    webview_root.join("profiles")
}

#[cfg(target_os = "windows")]
pub fn windows_profile_dir(webview_root: &Path, directory_name: &str) -> PathBuf {
    windows_profiles_dir(webview_root).join(directory_name)
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn parse_data_store_id(identifier_hex: &str) -> Result<[u8; 16], String> {
    let bytes = hex::decode(identifier_hex)
        .map_err(|err| format!("invalid data store identifier hex: {err}"))?;

    if bytes.len() != 16 {
        return Err(format!(
            "invalid data store identifier length {}, expected 16 bytes",
            bytes.len()
        ));
    }

    let mut out = [0_u8; 16];
    out.copy_from_slice(&bytes);
    Ok(out)
}
