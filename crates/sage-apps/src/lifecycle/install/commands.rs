use std::{fs, io};

use tauri::{State, command};

use crate::host::{AppState, Result};
use crate::lifecycle::{apps_root, list_installed_apps_internal};
use crate::types::ListedSageAppView;

#[command]
#[specta::specta]
pub async fn list_installed_apps(state: State<'_, AppState>) -> Result<Vec<ListedSageAppView>> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let root = apps_root(&base_path);

    fs::create_dir_all(&root).map_err(|err| {
        io::Error::other(format!(
            "failed to create apps directory {}: {err}",
            root.display()
        ))
    })?;

    list_installed_apps_internal(&root)
        .map(|apps| apps.iter().map(Into::into).collect())
        .map_err(|err| io::Error::other(format!("failed to list installed apps: {err}")).into())
}
