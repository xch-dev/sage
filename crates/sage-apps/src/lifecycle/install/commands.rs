use std::{fs, io};

use tauri::{State, command};

use crate::{
    AppState, AppsHostState, ListedSageAppView, Result, apps_root, list_installed_apps_internal,
};

#[command]
#[specta::specta]
pub async fn apps_list_installed_apps(
    state: State<'_, AppState>,
    apps_state: State<'_, AppsHostState>,
) -> Result<Vec<ListedSageAppView>> {
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

    list_installed_apps_internal(&apps_state.db)
        .await
        .map(|apps| apps.iter().map(Into::into).collect())
        .map_err(|err| io::Error::other(format!("failed to list installed apps: {err}")).into())
}
