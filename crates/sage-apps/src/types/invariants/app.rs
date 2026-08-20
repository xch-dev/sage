use std::path::PathBuf;

use crate::SageAppSnapshot;

pub fn validate_snapshot_entry_and_icon_exist(
    snapshot: &SageAppSnapshot,
    entry_file: &str,
    icon_file: Option<&str>,
    label: &str,
) -> anyhow::Result<()> {
    let entry_file = snapshot.file_path(entry_file);

    if !entry_file.is_file() {
        anyhow::bail!(
            "{label} entry file does not exist: {}",
            entry_file.display()
        );
    }

    if let Some(icon_file) = icon_file {
        let icon_file: PathBuf = snapshot.file_path(icon_file);

        if !icon_file.is_file() {
            anyhow::bail!("{label} icon file does not exist: {}", icon_file.display());
        }
    }

    Ok(())
}
