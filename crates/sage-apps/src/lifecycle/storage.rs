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

pub fn record_storage_cleanup_failure(
    base_path: &Path,
    app: &UserSageApp,
    error: &str,
) -> AnyResult<()> {
    let mut entries = read_pending_storage_cleanup_entries(base_path)?;

    let target = cleanup_target_from_storage(&app.common.storage);

    if let Some(entry) = entries.iter_mut().find(|entry| entry.target() == &target) {
        entry.record_failed_attempt(error);
    } else {
        entries.push(PendingStorageCleanupEntry::new(app, target, error));
    }

    write_pending_storage_cleanup_entries(base_path, &entries)
}

pub async fn process_pending_storage_cleanup(
    app: &AppHandle,
    base_path: &Path,
) -> AnyResult<()> {
    let entries = read_pending_storage_cleanup_entries(base_path)?;
    if entries.is_empty() {
        return Ok(());
    }

    let mut remaining = Vec::new();

    for mut entry in entries {
        match clear_app_storage_by_target(app, entry.target()).await {
            Ok(()) => {}
            Err(err) => {
                entry.record_failed_attempt(&err);
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
        .find(|entry| entry.origin_id() == app.common.origin_id)
    {
        existing.refresh_from_app(app, cleanup_pending);
    } else {
        entries.push(RetiredAppOriginEntry::new(app, cleanup_pending));
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
    use crate::lifecycle::{read_pending_storage_cleanup_entries, read_retired_app_origins};
    use crate::types::{
        PendingStorageCleanupTarget, SageAppCommon, SageAppManifestFile, SageAppPackageManifest,
        SageAppPackageManifestParts, SageAppSnapshot, SageGrantedPermissions,
        SageRequestedPermissions, UserSageApp, UserSageAppSource,
    };
    use tempfile::tempdir;

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile {
            path: path.to_string(),
            sha256: "a".repeat(64),
            size,
        }
    }

    fn sample_app(storage: InstalledSageAppStorage) -> UserSageApp {
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".into(),
            version: "1.0.0".into(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("index.html", 1)],
            entry: Some("index.html".into()),
            icon: Some("icon.png".into()),
            author: None,
            donation: None,
        })
            .unwrap();

        let granted_permissions =
            SageGrantedPermissions::new(manifest.permissions(), [], []).unwrap();

        let snapshot = SageAppSnapshot {
            manifest_hash: "hash".into(),
            snapshot_dir: "/tmp/test-app".into(),
            total_bytes: 1,
            manifest: manifest.clone(),
        };

        let mut common = SageAppCommon::new(
            "url-abc123".into(),
            "origin-1".into(),
            "/tmp/test-app".into(),
            &manifest,
            granted_permissions,
            storage,
            snapshot,
        )
            .unwrap();

        common.capability_flags.storage_may_contain_secrets = true;

        UserSageApp {
            common,
            source: UserSageAppSource::Url {
                app_url: "https://example.com/app/".into(),
                manifest_url: "https://example.com/app/sage-manifest.json".into(),
            },
            pending_update: None,
        }
    }

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

    #[test]
    fn record_storage_cleanup_failure_creates_unmanaged_target() {
        let dir = tempdir().unwrap();
        let app = sample_app(InstalledSageAppStorage::Unmanaged);

        record_storage_cleanup_failure(dir.path(), &app, "boom").unwrap();

        let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].target(), &PendingStorageCleanupTarget::Unmanaged);
        assert_eq!(entries[0].attempt_count(), 1);
        assert_eq!(entries[0].last_error(), Some("boom"));
    }

    #[test]
    fn record_storage_cleanup_failure_creates_apple_target() {
        let dir = tempdir().unwrap();
        let app = sample_app(InstalledSageAppStorage::AppleDataStore {
            identifier_hex: "abc123".into(),
        });

        record_storage_cleanup_failure(dir.path(), &app, "boom").unwrap();

        let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].target(),
            &PendingStorageCleanupTarget::AppleDataStore {
                identifier_hex: "abc123".into(),
            }
        );
    }

    #[test]
    fn record_storage_cleanup_failure_creates_windows_target() {
        let dir = tempdir().unwrap();
        let app = sample_app(InstalledSageAppStorage::WindowsProfile {
            directory_name: "profile-1".into(),
        });

        record_storage_cleanup_failure(dir.path(), &app, "boom").unwrap();

        let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].target(),
            &PendingStorageCleanupTarget::WindowsProfile {
                directory_name: "profile-1".into(),
            }
        );
    }

    #[test]
    fn record_storage_cleanup_failure_updates_existing_entry_by_target_not_app_id() {
        let dir = tempdir().unwrap();

        let app_a = sample_app(InstalledSageAppStorage::Unmanaged);
        record_storage_cleanup_failure(dir.path(), &app_a, "first").unwrap();

        let mut app_b = sample_app(InstalledSageAppStorage::Unmanaged);
        app_b.common.id = "url-other".into();
        app_b.common.name = "Other App".into();

        record_storage_cleanup_failure(dir.path(), &app_b, "second").unwrap();

        let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].app_id(), "url-other");
        assert_eq!(entries[0].app_name(), "Other App");
        assert_eq!(entries[0].attempt_count(), 2);
        assert_eq!(entries[0].last_error(), Some("second"));
    }

    #[test]
    fn enqueue_retired_app_origin_ignores_zip_apps() {
        let dir = tempdir().unwrap();
        let mut app = sample_app(InstalledSageAppStorage::Unmanaged);
        app.source = UserSageAppSource::Zip;

        enqueue_retired_app_origin(dir.path(), &app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn enqueue_retired_app_origin_creates_new_entry_for_url_app() {
        let dir = tempdir().unwrap();
        let app = sample_app(InstalledSageAppStorage::Unmanaged);

        enqueue_retired_app_origin(dir.path(), &app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].app_id(), app.common.id);
        assert_eq!(entries[0].origin_id(), app.common.origin_id);
        assert!(entries[0].cleanup_pending());
        assert!(entries[0].storage_may_contain_secrets());
    }

    #[test]
    fn enqueue_retired_app_origin_updates_existing_origin_entry() {
        let dir = tempdir().unwrap();
        let app = sample_app(InstalledSageAppStorage::Unmanaged);

        enqueue_retired_app_origin(dir.path(), &app, true).unwrap();
        enqueue_retired_app_origin(dir.path(), &app, false).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].cleanup_pending());
    }

    #[test]
    fn enqueue_retired_app_origin_updates_secret_taint_flag() {
        let dir = tempdir().unwrap();
        let mut app = sample_app(InstalledSageAppStorage::Unmanaged);
        app.common.capability_flags.storage_may_contain_secrets = false;

        enqueue_retired_app_origin(dir.path(), &app, false).unwrap();

        app.common.capability_flags.storage_may_contain_secrets = true;
        enqueue_retired_app_origin(dir.path(), &app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].storage_may_contain_secrets());
        assert!(entries[0].cleanup_pending());
    }
}
