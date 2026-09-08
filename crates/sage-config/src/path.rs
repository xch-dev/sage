use std::{env, path::PathBuf};

pub const SAGE_ROOT_ENV: &str = "SAGE_ROOT";

const SAGE_DATA_DIRECTORY: &str = "com.rigidnetwork.sage";

pub fn sage_root_override() -> Option<PathBuf> {
    env::var_os(SAGE_ROOT_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub fn sage_root() -> Option<PathBuf> {
    resolve_sage_root(sage_root_override(), dirs::data_dir())
}

fn resolve_sage_root(root: Option<PathBuf>, data_dir: Option<PathBuf>) -> Option<PathBuf> {
    root.or_else(|| data_dir.map(|path| path.join(SAGE_DATA_DIRECTORY)))
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
