use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::lifecycle::apps_root;

const APPS_SETTINGS_FILE: &str = ".sage-apps-settings.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SageAppsSettings {
    #[serde(default)]
    pub auto_update_enabled: bool,
}

pub fn apps_settings_path(base_path: &Path) -> PathBuf {
    apps_root(base_path).join(APPS_SETTINGS_FILE)
}

pub fn read_apps_settings(base_path: &Path) -> anyhow::Result<SageAppsSettings> {
    let path = apps_settings_path(base_path);

    if !path.exists() {
        return Ok(SageAppsSettings::default());
    }

    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))
}

pub fn write_apps_settings(base_path: &Path, settings: &SageAppsSettings) -> anyhow::Result<()> {
    let root = apps_root(base_path);
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create apps root {}", root.display()))?;

    let path = apps_settings_path(base_path);
    let tmp = path.with_extension("json.tmp");

    let text = serde_json::to_string_pretty(settings)
        .context("failed to serialize Sage apps settings")?;

    fs::write(&tmp, format!("{text}\n"))
        .with_context(|| format!("failed to write {}", tmp.display()))?;

    fs::rename(&tmp, &path).with_context(|| {
        format!(
            "failed to move {} to {}",
            tmp.display(),
            path.display()
        )
    })?;

    Ok(())
}
