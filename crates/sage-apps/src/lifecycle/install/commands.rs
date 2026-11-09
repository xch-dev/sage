use std::{fs, io};

use tauri::{AppHandle, State, command};

use crate::{AppsHostState, ListedSageAppView, Result, apps_root, list_installed_apps_internal};

#[command]
#[specta::specta]
pub async fn apps_list_installed_apps(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
) -> Result<Vec<ListedSageAppView>> {
    let root = apps_root(&apps_state.root);

    fs::create_dir_all(&root).map_err(|err| {
        io::Error::other(format!(
            "failed to create apps directory {}: {err}",
            root.display()
        ))
    })?;

    list_installed_apps_internal(&apps_state.db)
        .await
        .map(|apps| {
            apps.iter()
                .map(|app| {
                    ListedSageAppView::from_listed_with_current_version(
                        app,
                        &app_handle.package_info().version,
                    )
                })
                .collect()
        })
        .map_err(|err| io::Error::other(format!("failed to list installed apps: {err}")).into())
}
