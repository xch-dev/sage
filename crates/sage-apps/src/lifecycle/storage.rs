use std::path::Path;

#[cfg(target_os = "windows")]
use anyhow::Context;
use anyhow::{Result as AnyResult, anyhow};
#[cfg(target_os = "windows")]
use std::fs;
use tauri::{AppHandle, command};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use uuid::Uuid;

use crate::lifecycle::{
    read_pending_storage_cleanup_entries, read_retired_app_origins,
    write_pending_storage_cleanup_entries, write_retired_app_origins,
};
use crate::runtime::{resolve_stopped_app, run_verified_storage_clear_cycle};
use crate::storage::{cleanup_target_from_storage, parse_data_store_id};
use crate::types::{
    InstalledSageAppStorage, PendingStorageCleanupEntry, PendingStorageCleanupTarget,
    RetiredAppOriginEntry, SharedSageApp,
};

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
    app: &SharedSageApp,
    error: &str,
) -> AnyResult<()> {
    let mut entries = read_pending_storage_cleanup_entries(base_path)?;

    let target = cleanup_target_from_storage(&app.with(|app| app.storage().clone()));

    if let Some(entry) = entries.iter_mut().find(|entry| entry.target() == &target) {
        entry.record_failed_attempt(error);
    } else {
        entries.push(PendingStorageCleanupEntry::new(app, target, error));
    }

    write_pending_storage_cleanup_entries(base_path, &entries)
}

pub async fn process_pending_storage_cleanup(app: &AppHandle, base_path: &Path) -> AnyResult<()> {
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

pub fn enqueue_retired_app_origin(app: &SharedSageApp, cleanup_pending: bool) -> AnyResult<()> {
    let base_path = app.with(|app| {
        let user = app.as_user()?;

        match user.source() {
            crate::types::UserSageAppSource::Url { .. } => {}
            crate::types::UserSageAppSource::Zip => return None,
        }

        let app_path = app.app_path();

        app_path
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
    });

    let Some(base_path) = base_path else {
        return Ok(());
    };

    let mut entries = read_retired_app_origins(&base_path)?;

    if let Some(existing) = entries
        .iter_mut()
        .find(|entry| entry.origin_id() == app.origin_id())
    {
        existing.update_retirement_state(app, cleanup_pending);
    } else {
        entries.push(RetiredAppOriginEntry::new(app, cleanup_pending));
    }

    write_retired_app_origins(&base_path, &entries)
}

#[command]
#[specta::specta]
pub async fn apps_clear_runtime_browsing_data(
    app_handle: AppHandle,
    app_id: String,
) -> Result<(), String> {
    let resolved_app = resolve_stopped_app(&app_handle, &app_id)
        .await
        .map_err(|e| e.to_string())?;

    run_verified_storage_clear_cycle(&app_handle, &resolved_app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::capabilities::list::UserBridgeCapability;
    use crate::lifecycle::{
        app_dir, read_pending_storage_cleanup_entries, read_retired_app_origins,
    };
    use crate::runtime::{SageAppRuntimeMode, SageAppRuntimeRecord, SageAppRuntimeVisibility};
    use crate::types::{
        AppPresentation, SageAppCommon, SageAppIdentity, SageAppManifestFile,
        SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot, SageAppWalletScope,
        SageGrantedPermissions, SageRequestedCapabilities, SageRequestedPermissions, SharedSageApp,
        UserSageApp, UserSageAppSource,
    };
    use tempfile::tempdir;

    fn write_index(app_dir: &Path) {
        std::fs::create_dir_all(app_dir).unwrap();
        std::fs::write(app_dir.join("index.html"), "x").unwrap();
    }

    fn sample_app_with(
        base_path: &Path,
        app_id: &str,
        name: &str,
        storage: InstalledSageAppStorage,
        source: UserSageAppSource,
        storage_may_contain_secrets: bool,
    ) -> SharedSageApp {
        let app_dir = app_dir(base_path, app_id);
        write_index(&app_dir);

        let requested_permissions = SageRequestedPermissions::new(
            crate::types::SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::StoragePersistentWebview],
                [UserBridgeCapability::WalletGetSecretKey],
            ),
        )
        .unwrap();

        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: name.to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: requested_permissions.clone(),
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 1).unwrap()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap();

        let granted_capabilities = if storage_may_contain_secrets {
            vec![
                UserBridgeCapability::StoragePersistentWebview,
                UserBridgeCapability::WalletGetSecretKey,
            ]
        } else {
            vec![UserBridgeCapability::StoragePersistentWebview]
        };

        let granted_permissions =
            SageGrantedPermissions::new(&requested_permissions, granted_capabilities, []).unwrap();

        let snapshot =
            SageAppSnapshot::new("hash", app_dir.to_string_lossy().to_string(), manifest).unwrap();

        let mut common = SageAppCommon::new(
            SageAppIdentity::new(app_id, app_id, app_dir.to_string_lossy().to_string()).unwrap(),
            granted_permissions,
            storage,
            snapshot,
            SageAppWalletScope::AllWallets,
        )
        .unwrap();

        if storage_may_contain_secrets {
            common.mark_storage_may_contain_secrets();
        }

        let app = SharedSageApp::new(UserSageApp::new_installed(common, source).into_sage_app());

        if storage_may_contain_secrets {
            let _record = SageAppRuntimeRecord::new(
                &app,
                "test",
                "sage-app://test/index.html",
                AppPresentation::Taskbar,
                SageAppRuntimeMode::Inline,
                SageAppRuntimeVisibility::Visible,
                false,
            )
            .unwrap();
        }

        assert!(app.is_user_app(), "sample app should remain a user app");

        app
    }

    fn sample_app(base_path: &Path, storage: InstalledSageAppStorage) -> SharedSageApp {
        sample_app_with(
            base_path,
            "url-abc123",
            "Test App",
            storage,
            UserSageAppSource::url("https://example.com/app/").unwrap(),
            true,
        )
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
        let app = sample_app(dir.path(), InstalledSageAppStorage::Unmanaged);

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
        let app = sample_app(
            dir.path(),
            InstalledSageAppStorage::AppleDataStore {
                identifier_hex: "abc123".into(),
            },
        );

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
        let app = sample_app(
            dir.path(),
            InstalledSageAppStorage::WindowsProfile {
                directory_name: "profile-1".into(),
            },
        );

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
    fn record_storage_cleanup_failure_merges_existing_entry_by_target() {
        let dir = tempdir().unwrap();

        let app_a = sample_app(dir.path(), InstalledSageAppStorage::Unmanaged);
        record_storage_cleanup_failure(dir.path(), &app_a, "first").unwrap();

        let app_b = sample_app_with(
            dir.path(),
            "url-other",
            "Other App",
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::url("https://example.com/other/").unwrap(),
            true,
        );

        record_storage_cleanup_failure(dir.path(), &app_b, "second").unwrap();

        let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].app_id(), "url-abc123");
        assert_eq!(entries[0].app_name(), "Test App");
        assert_eq!(entries[0].attempt_count(), 2);
        assert_eq!(entries[0].last_error(), Some("second"));
    }

    #[test]
    fn enqueue_retired_app_origin_ignores_zip_apps() {
        let dir = tempdir().unwrap();
        let app = sample_app_with(
            dir.path(),
            "url-abc123",
            "Test App",
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::Zip,
            true,
        );

        enqueue_retired_app_origin(&app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn enqueue_retired_app_origin_creates_new_entry_for_url_app() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), InstalledSageAppStorage::Unmanaged);

        enqueue_retired_app_origin(&app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].app_id(), app.id());
        assert_eq!(entries[0].origin_id(), app.origin_id());
        assert!(entries[0].cleanup_pending());
        assert!(entries[0].storage_may_contain_secrets());
    }

    #[test]
    fn enqueue_retired_app_origin_updates_existing_origin_entry() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), InstalledSageAppStorage::Unmanaged);

        enqueue_retired_app_origin(&app, true).unwrap();
        enqueue_retired_app_origin(&app, false).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].cleanup_pending());
    }

    #[test]
    fn enqueue_retired_app_origin_updates_secret_taint_flag() {
        let dir = tempdir().unwrap();

        let clean_app = sample_app_with(
            dir.path(),
            "url-abc123",
            "Test App",
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::url("https://example.com/app/").unwrap(),
            false,
        );

        enqueue_retired_app_origin(&clean_app, false).unwrap();

        let tainted_app = sample_app_with(
            dir.path(),
            "url-abc123",
            "Test App",
            InstalledSageAppStorage::Unmanaged,
            UserSageAppSource::url("https://example.com/app/").unwrap(),
            true,
        );

        enqueue_retired_app_origin(&tainted_app, true).unwrap();

        let entries = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].storage_may_contain_secrets());
        assert!(entries[0].cleanup_pending());
    }
}
