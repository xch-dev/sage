use std::{
    env,
    path::{self, PathBuf},
};

pub const SAGE_ROOT_ENV: &str = "SAGE_ROOT";

/// Must match the `identifier` in `src-tauri/tauri.conf.json`, since the desktop app resolves its
/// default root from the Tauri app data directory.
pub const SAGE_APP_IDENTIFIER: &str = "com.rigidnetwork.sage";

/// Returns the absolute root set by `SAGE_ROOT`, if any.
///
/// Relative values are resolved against the current directory up front, so that every consumer
/// (including webviews, which resolve relative paths on their own) agrees on the location.
pub fn sage_root_override() -> Option<PathBuf> {
    env::var_os(SAGE_ROOT_ENV)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let root = PathBuf::from(value);
            path::absolute(&root).unwrap_or(root)
        })
}

pub fn sage_root() -> Option<PathBuf> {
    resolve_sage_root(sage_root_override(), dirs::data_dir())
}

fn resolve_sage_root(root: Option<PathBuf>, data_dir: Option<PathBuf>) -> Option<PathBuf> {
    root.or_else(|| data_dir.map(|path| path.join(SAGE_APP_IDENTIFIER)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn environment_root_overrides_data_directory() {
        let root = resolve_sage_root(
            Some(Path::new("/custom/sage").to_path_buf()),
            Some(Path::new("/data").to_path_buf()),
        );

        assert_eq!(root, Some(Path::new("/custom/sage").to_path_buf()));
    }

    #[test]
    fn default_root_uses_application_identifier() {
        let root = resolve_sage_root(None, Some(Path::new("/data").to_path_buf()));

        assert_eq!(root, Some(Path::new("/data").join("com.rigidnetwork.sage")));
    }
}
