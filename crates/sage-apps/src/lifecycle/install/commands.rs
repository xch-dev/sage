use std::{fs, io, path::Path};

use tauri::{AppHandle, State, command};

use crate::host::{AppState, Result};
use crate::lifecycle::install::install_app_from_source;
use crate::lifecycle::install::url::{UrlInstallSource};
use crate::lifecycle::install::zip::ZipInstallSource;
use crate::lifecycle::{
    apps_root, list_installed_apps_internal, read_manifest, unzip_to_dir,
    validate_package_structure,
};
use crate::types::{
    ListedSageApp, SageAppPackageManifest, SageAppUrlPreview, SageGrantedPermissions, UserSageApp,
};
use uuid::Uuid;

#[allow(clippy::needless_pass_by_value)]
#[command]
#[specta::specta]
pub fn preview_app_zip(zip_path: String) -> Result<SageAppPackageManifest> {
    let unpack_dir = std::env::temp_dir().join(format!(".sage-preview-{}", Uuid::new_v4()));

    let result = (|| -> anyhow::Result<SageAppPackageManifest> {
        unzip_to_dir(Path::new(&zip_path), &unpack_dir)?;
        let package_root = crate::lifecycle::detect_package_root(&unpack_dir)?;
        let manifest = read_manifest(&package_root)?;
        validate_package_structure(&package_root)?;

        Ok(manifest)
    })();

    let _ = fs::remove_dir_all(&unpack_dir);

    result.map_err(|err| {
        io::Error::other(format!("failed to preview app zip {zip_path}: {err}")).into()
    })
}

#[command]
#[specta::specta]
pub async fn preview_app_url(app_url: String) -> Result<SageAppUrlPreview> {
    SageAppUrlPreview::new(app_url).await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")).into())
}

#[command]
#[specta::specta]
pub async fn list_installed_apps(state: State<'_, AppState>) -> Result<Vec<ListedSageApp>> {
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
        .map_err(|err| io::Error::other(format!("failed to list installed apps: {err}")).into())
}

#[command]
#[specta::specta]
pub async fn install_app_zip(
    app: AppHandle,
    state: State<'_, AppState>,
    zip_path: String,
    granted_permissions: SageGrantedPermissions,
) -> Result<UserSageApp> {
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

    let source = ZipInstallSource::new(&root, zip_path.clone());
    let unpack_dir = source.unpack_dir.clone();

    let result = install_app_from_source(&app, &base_path, granted_permissions, source).await;

    if unpack_dir.exists() {
        let _ = fs::remove_dir_all(&unpack_dir);
    }

    result.map_err(|err| {
        io::Error::other(format!("failed to install app zip {zip_path}: {err}")).into()
    })
}

#[command]
#[specta::specta]
pub async fn install_app_url(
    app: AppHandle,
    state: State<'_, AppState>,
    app_url: String,
    granted_permissions: SageGrantedPermissions,
) -> Result<UserSageApp> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    install_app_from_source(
        &app,
        &base_path,
        granted_permissions,
        UrlInstallSource {
            app_url: app_url.clone(),
        },
    )
    .await
    .map_err(|err| io::Error::other(format!("failed to install app URL {app_url}: {err}")).into())
}
