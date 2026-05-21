use anyhow::{Result as AnyResult};
use tauri::{AppHandle, command, State, Manager};
use std::path::Path;
use uuid::Uuid;
#[cfg(target_os = "windows")]
use {
    std::fs,
    anyhow::Context,
};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use {
    anyhow::anyhow,
    crate::storage::parse_data_store_id,
};
use crate::AppsHostState;
use crate::runtime::{resolve_stopped_app};
use crate::types::{SageAppStorage, SharedSageApp};

pub struct RegisteredSageAppStorage {
    pub storage_id: i64,
    pub storage: SageAppStorage,
}

#[command]
#[specta::specta]
pub async fn apps_clear_runtime_browsing_data(
    app_handle: AppHandle,
    app_id: String,
) -> Result<(), String> {
    let apps_state: State<'_, AppsHostState> = app_handle.state();

    let resolved_app = resolve_stopped_app(&app_handle, &app_id)
        .await
        .map_err(|e| e.to_string())?;

    rotate_stopped_app_storage_and_origin(&app_handle, &apps_state, &resolved_app).await
}

pub async fn allocate_new_storage(
    app: &AppHandle,
    host_state: &State<'_, AppsHostState>,
    base_path: &Path,
) -> AnyResult<RegisteredSageAppStorage> {
    let storage = allocate_new_os_storage(app, base_path).await?;
    let storage_id = host_state.db.register_storage(&storage).await?;

    Ok(RegisteredSageAppStorage {
        storage_id,
        storage,
    })
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub async fn allocate_new_os_storage(
    app: &AppHandle,
    _base_path: &Path,
) -> AnyResult<SageAppStorage> {
    loop {
        let identifier = *Uuid::new_v4().as_bytes();
        let existing_ids = app
            .fetch_data_store_identifiers()
            .await
            .map_err(|err| anyhow!("failed to fetch data store identifiers: {err}"))?;

        if existing_ids.iter().all(|existing| *existing != identifier) {
            return Ok(SageAppStorage::AppleDataStore {
                identifier_hex: hex::encode(identifier),
            });
        }
    }
}

#[cfg(target_os = "windows")]
pub async fn allocate_new_os_storage(
    _app: &AppHandle,
    base_path: &Path,
) -> AnyResult<SageAppStorage> {
    let profiles_root = base_path.join("profiles");
    fs::create_dir_all(&profiles_root).with_context(|| {
        format!(
            "failed to create profiles directory {}",
            profiles_root.display()
        )
    })?;

    loop {
        let directory_name = format!("profile-{}", Uuid::new_v4());
        let candidate = profiles_root.join(&directory_name);

        if !candidate.exists() {
            return Ok(SageAppStorage::WindowsProfile { directory_name });
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
pub async fn allocate_new_os_storage(
    _app: &AppHandle,
    _base_path: &Path,
) -> AnyResult<SageAppStorage> {
    Ok(SageAppStorage::Unmanaged)
}

pub async fn process_pending_storage_cleanup(app: &AppHandle, _base_path: &Path) -> AnyResult<()> {
    let host_state: State<'_, AppsHostState> = app.state();

    for abandoned in host_state.db.list_abandoned_managed_storages().await? {
        clear_app_storage_by_target(app, &abandoned.storage)
            .await
            .map_err(anyhow::Error::msg)?;

        host_state
            .db
            .delete_origins_for_abandoned_storage(abandoned.id)
            .await?;

        host_state
            .db
            .delete_abandoned_storage(abandoned.id)
            .await?;
    }

    Ok(())
}

pub async fn clear_app_storage_by_target(
    app: &AppHandle,
    target: &SageAppStorage,
) -> Result<(), String> {
    match target {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        SageAppStorage::AppleDataStore { identifier_hex } => {
            let target_id = parse_data_store_id(identifier_hex)?;
            let existing_ids = app
                .fetch_data_store_identifiers()
                .await
                .map_err(|e| format!("failed to fetch data store identifiers: {e}"))?;

            if existing_ids.contains(&target_id) {
                app.remove_data_store(target_id)
                    .await
                    .map_err(|e| format!("failed to remove data store: {e}"))?;
            }
        }

        #[cfg(target_os = "windows")]
        SageAppStorage::WindowsProfile { directory_name } => {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

            let profile_dir = app_data_dir.join(crate::storage::data_directory_for(directory_name));

            match fs::remove_dir_all(&profile_dir) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(format!(
                        "failed to remove profile dir {}: {err}",
                        profile_dir.display()
                    ));
                }
            }
        }

        SageAppStorage::Unmanaged => {}

        #[allow(unreachable_patterns)]
        _ => {}
    }

    Ok(())
}

pub async fn rotate_stopped_app_storage_and_origin(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    resolved_app: &crate::types::ResolvedStoppedApp,
) -> Result<(), String> {
    let app = resolved_app.with_app(SharedSageApp::clone);

    rotate_app_storage_and_origin(app_handle, apps_state, &app).await
}

pub(crate) async fn rotate_app_storage_and_origin(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app: &SharedSageApp,
) -> Result<(), String> {
    let app_id = app.id();

    let (
        previous_storage,
        previous_origin_id,
        previous_origin_webview_storage_may_contain_secrets,
    ) = app.with(|app| {
        (
            app.storage().clone(),
            app.origin_id().to_string(),
            app.common().origin_webview_storage_may_contain_secrets(),
        )
    });

    let base_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?;

    let next_storage = allocate_new_storage(app_handle, apps_state, &base_path)
        .await
        .map_err(|err| format!("failed to allocate rotated storage: {err}"))?;

    let next_origin_id = crate::lifecycle::install::fresh_origin_id(&app_id);

    let origin_row_id = apps_state
        .db
        .register_origin(&next_origin_id, next_storage.storage_id)
        .await
        .map_err(|err| format!("failed to register rotated app origin: {err}"))?;

    app.try_mutate(|sage_app| {
        sage_app.common_mut().replace_storage_and_origin(
            next_storage.storage.clone(),
            next_origin_id.clone(),
            false,
        )?;

        Ok::<_, anyhow::Error>(())
    })
        .map_err(|err| format!("failed to persist rotated app storage/origin: {err}"))?;

    if let Err(err) = apps_state
        .db
        .update_app_assignment(&app_id, next_storage.storage_id, origin_row_id)
        .await
    {
        let rollback_result = app.try_mutate(|sage_app| {
            sage_app
                .common_mut()
                .replace_storage_and_origin(
                    previous_storage,
                    previous_origin_id,
                    previous_origin_webview_storage_may_contain_secrets,
                )?;

            Ok::<_, anyhow::Error>(())
        });

        return Err(match rollback_result {
            Ok(()) => format!("failed to update rotated app assignment: {err}"),
            Err(rollback_err) => format!(
                "failed to update rotated app assignment: {err}; also failed to roll back app metadata: {rollback_err}"
            ),
        });
    }

    Ok(())
}
