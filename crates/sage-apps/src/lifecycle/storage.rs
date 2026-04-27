use std::io;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::path::{Path, PathBuf};

use crate::AppsHostState;
use crate::host::AppState;
use crate::lifecycle::flags::mark_storage_may_contain_secrets;
use crate::lifecycle::{
    read_installed_app_by_id, read_pending_storage_cleanup_entries, read_retired_app_origins,
    write_installed_app_metadata, write_pending_storage_cleanup_entries, write_retired_app_origins,
};
use crate::runtime::resolve_app;
use crate::runtime::stop::close_runtime_internal;
use crate::storage::{cleanup_target_from_storage, parse_data_store_id};
use crate::types::{
    InstalledSageAppStorage, PendingStorageCleanupEntry, PendingStorageCleanupTarget,
    RetiredAppOriginEntry, UserSageApp, UserSageAppSource,
};
use crate::utils::unix_timestamp_ms;
use anyhow::{Result as AnyResult, anyhow};
use tauri::{AppHandle, Manager, State, command};
use uuid::Uuid;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub async fn allocate_new_storage(
    app: &AppHandle,
    _base_path: &Path,
) -> AnyResult<InstalledSageAppStorage> {
    loop {
        let identifier = *Uuid::new_v4().as_bytes();
        let existing_ids = app
            .fetch_data_store_identifiers()
            .await
            .map_err(|err| anyhow!("failed to fetch data store identifiers: {err}"))?;

        if existing_ids.iter().all(|existing| *existing != identifier) {
            return Ok(InstalledSageAppStorage::AppleDataStore {
                identifier_hex: hex::encode(identifier),
            });
        }
    }
}

#[cfg(target_os = "windows")]
pub async fn allocate_new_storage(
    _app: &AppHandle,
    base_path: &Path,
) -> AnyResult<InstalledSageAppStorage> {
    let profiles_root = base_path.join("profiles");
    fs::create_dir_all(&profiles_root).with_context(|| {
        format!(
            "failed to create profiles directory {}",
            profiles_root.display()
        )
    })?;

    loop {
        let directory_name = format!("profile-{}", uuid::Uuid::new_v4());
        let candidate = profiles_root.join(&directory_name);

        if !candidate.exists() {
            return Ok(InstalledSageAppStorage::WindowsProfile { directory_name });
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
pub async fn allocate_new_storage(
    _app: &AppHandle,
    _base_path: &Path,
) -> AnyResult<InstalledSageAppStorage> {
    Ok(InstalledSageAppStorage::Unmanaged)
}

pub fn enqueue_pending_storage_cleanup(
    base_path: &Path,
    app: &UserSageApp,
    error: &str,
) -> AnyResult<()> {
    let mut entries = read_pending_storage_cleanup_entries(base_path)?;

    let target = cleanup_target_from_storage(&app.common.storage);
    let existing = entries.iter_mut().find(|entry| entry.target == target);

    let now = unix_timestamp_ms();
    match existing {
        Some(entry) => {
            entry.last_attempt_at_ms = Some(now);
            entry.attempt_count = entry.attempt_count.saturating_add(1);
            entry.last_error = Some(error.to_string());
            entry.app_id = app.common.id.clone();
            entry.app_name = app.common.name.clone();
        }
        None => entries.push(PendingStorageCleanupEntry {
            id: Uuid::new_v4().to_string(),
            app_id: app.common.id.clone(),
            app_name: app.common.name.clone(),
            target,
            created_at_ms: now,
            last_attempt_at_ms: Some(now),
            attempt_count: 1,
            last_error: Some(error.to_string()),
        }),
    }

    write_pending_storage_cleanup_entries(base_path, &entries)
}

pub async fn retry_pending_storage_cleanup(app: &AppHandle, base_path: &Path) -> AnyResult<()> {
    let entries = read_pending_storage_cleanup_entries(base_path)?;
    if entries.is_empty() {
        return Ok(());
    }

    let mut remaining = Vec::new();

    for mut entry in entries {
        entry.last_attempt_at_ms = Some(unix_timestamp_ms());
        entry.attempt_count = entry.attempt_count.saturating_add(1);

        match clear_app_storage_by_target(app, &entry.target).await {
            Ok(()) => {}
            Err(err) => {
                entry.last_error = Some(err);
                remaining.push(entry);
            }
        }
    }

    write_pending_storage_cleanup_entries(base_path, &remaining)
}

pub async fn clear_app_storage_by_target(
    app: &AppHandle,
    target: &PendingStorageCleanupTarget,
) -> Result<(), String> {
    match target {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        PendingStorageCleanupTarget::AppleDataStore { identifier_hex } => {
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
        PendingStorageCleanupTarget::WindowsProfile { directory_name } => {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

            let profile_dir = app_data_dir.join(crate::storage::data_directory_for(directory_name));

            match std::fs::remove_dir_all(&profile_dir) {
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

        PendingStorageCleanupTarget::Unmanaged => {}

        #[allow(unreachable_patterns)]
        _ => {}
    }

    Ok(())
}

pub fn enqueue_retired_app_origin(
    base_path: &Path,
    app: &UserSageApp,
    cleanup_pending: bool,
) -> AnyResult<()> {
    let UserSageAppSource::Url { .. } = &app.source else {
        return Ok(());
    };

    let mut entries = read_retired_app_origins(base_path)?;

    if let Some(existing) = entries
        .iter_mut()
        .find(|entry| entry.origin_id == app.common.origin_id)
    {
        existing.app_id = app.common.id.clone();
        existing.app_name = app.common.name.clone();
        existing.cleanup_pending = cleanup_pending;
        existing.storage_may_contain_secrets =
            app.common.capability_flags.storage_may_contain_secrets;
    } else {
        entries.push(RetiredAppOriginEntry {
            id: Uuid::new_v4().to_string(),
            app_id: app.common.id.clone(),
            app_name: app.common.name.clone(),
            origin_id: app.common.origin_id.clone(),
            created_at_ms: unix_timestamp_ms(),
            storage_may_contain_secrets: app.common.capability_flags.storage_may_contain_secrets,
            cleanup_pending,
        });
    }

    write_retired_app_origins(base_path, &entries)
}

pub async fn clear_runtime_browsing_data_internal(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<(), String> {
    let _ = close_runtime_internal(app, apps_state, app_id).await;
    apps_clear_runtime_browsing_data(app.clone(), app_id.to_string()).await
}

#[command]
#[specta::specta]
pub async fn apps_clear_runtime_browsing_data(
    app: AppHandle,
    app_id: String,
) -> Result<(), String> {
    let resolved = resolve_app(&app, &app_id)?;

    close_runtime_internal(&app, &app.state(), &app_id).await?;

    let target = cleanup_target_from_storage(resolved.storage());
    clear_app_storage_by_target(&app, &target).await
}

#[command]
#[specta::specta]
pub async fn apps_mark_storage_may_contain_secrets(
    state: State<'_, AppState>,
    app_id: String,
) -> crate::host::Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let mut app = read_installed_app_by_id(&base_path, &app_id)
        .map_err(|err| io::Error::other(format!("failed to read app {app_id}: {err}")))?;

    if !app.common.capability_flags.has_secret_access {
        return Ok(());
    }

    if app.common.capability_flags.storage_may_contain_secrets {
        return Ok(());
    }

    app.common.capability_flags = mark_storage_may_contain_secrets(&app.common.capability_flags);

    let app_dir = PathBuf::from(&app.common.app_dir);
    write_installed_app_metadata(&app, &app_dir)
        .map_err(|err| io::Error::other(format!("failed to write metadata: {err}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{InstalledSageAppStorage, PendingStorageCleanupTarget};

    #[test]
    fn target_from_storage_maps_apple_data_store() {
        let target = cleanup_target_from_storage(&InstalledSageAppStorage::AppleDataStore {
            identifier_hex: "abc123".into(),
        });

        assert_eq!(
            target,
            PendingStorageCleanupTarget::AppleDataStore {
                identifier_hex: "abc123".into(),
            }
        );
    }

    #[test]
    fn target_from_storage_maps_windows_profile() {
        let target = cleanup_target_from_storage(&InstalledSageAppStorage::WindowsProfile {
            directory_name: "profile-1".into(),
        });

        assert_eq!(
            target,
            PendingStorageCleanupTarget::WindowsProfile {
                directory_name: "profile-1".into(),
            }
        );
    }

    #[test]
    fn target_from_storage_maps_unmanaged() {
        let target = cleanup_target_from_storage(&InstalledSageAppStorage::Unmanaged);
        assert_eq!(target, PendingStorageCleanupTarget::Unmanaged);
    }
}
