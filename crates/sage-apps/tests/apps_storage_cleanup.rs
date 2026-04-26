mod common;

use common::{empty_permissions, sample_manifest_file};
use sage_apps::lifecycle::storage::{
    enqueue_pending_storage_cleanup, enqueue_retired_app_origin,
};
use sage_apps::lifecycle::{
    read_pending_storage_cleanup_entries, read_retired_app_origins,
};
use sage_apps::types::{
    InstalledSageAppStorage, PendingStorageCleanupTarget, SageAppFlags,
    SageAppCommon, SageAppPackageManifest, SageAppSnapshot,
    SageGrantedNetworkPermissions, SageGrantedPermissions, UserSageApp,
    UserSageAppSource,
};
use tempfile::tempdir;

fn sample_app(storage: InstalledSageAppStorage) -> UserSageApp {
    UserSageApp {
        common: SageAppCommon {
            id: "url-abc123".into(),
            origin_id: "origin-1".into(),
            name: "Test App".into(),
            version: "1.0.0".into(),
            app_dir: "/tmp/test-app".into(),
            entry_file: "index.html".into(),
            icon_file: "icon.png".into(),
            requested_permissions: empty_permissions(),
            granted_permissions: SageGrantedPermissions {
                capabilities: vec![],
                network: SageGrantedNetworkPermissions { whitelist: vec![] },
            },
            capability_flags: SageAppFlags {
                has_secret_access: false,
                has_external_access: false,
                storage_may_contain_secrets: true,
                isolated: false,
            },
            storage,
            active_snapshot: SageAppSnapshot {
                manifest_hash: "hash".into(),
                snapshot_dir: "/tmp/test-app".into(),
                total_bytes: 1,
                manifest: SageAppPackageManifest {
                    name: "Test App".into(),
                    version: "1.0.0".into(),
                    permissions: empty_permissions(),
                    files: vec![sample_manifest_file("index.html", 1)],
                    entry: Some("index.html".into()),
                    icon: Some("icon.png".into()),
                    author: None,
                    donation: None,
                },
            },
        },
        source: UserSageAppSource::Url {
            app_url: "https://example.com/app/".into(),
            manifest_url: "https://example.com/app/sage-manifest.json".into(),
        },
        pending_update: None,
    }
}

#[test]
fn enqueue_pending_storage_cleanup_creates_unmanaged_target() {
    let dir = tempdir().unwrap();
    let app = sample_app(InstalledSageAppStorage::Unmanaged);

    enqueue_pending_storage_cleanup(dir.path(), &app, "boom").unwrap();

    let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].target, PendingStorageCleanupTarget::Unmanaged);
    assert_eq!(entries[0].attempt_count, 1);
    assert_eq!(entries[0].last_error.as_deref(), Some("boom"));
}

#[test]
fn enqueue_pending_storage_cleanup_creates_apple_target() {
    let dir = tempdir().unwrap();
    let app = sample_app(InstalledSageAppStorage::AppleDataStore {
        identifier_hex: "abc123".into(),
    });

    enqueue_pending_storage_cleanup(dir.path(), &app, "boom").unwrap();

    let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].target,
        PendingStorageCleanupTarget::AppleDataStore {
            identifier_hex: "abc123".into(),
        }
    );
}

#[test]
fn enqueue_pending_storage_cleanup_creates_windows_target() {
    let dir = tempdir().unwrap();
    let app = sample_app(InstalledSageAppStorage::WindowsProfile {
        directory_name: "profile-1".into(),
    });

    enqueue_pending_storage_cleanup(dir.path(), &app, "boom").unwrap();

    let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].target,
        PendingStorageCleanupTarget::WindowsProfile {
            directory_name: "profile-1".into(),
        }
    );
}

#[test]
fn enqueue_pending_storage_cleanup_updates_existing_entry_by_target_not_app_id() {
    let dir = tempdir().unwrap();

    let app_a = sample_app(InstalledSageAppStorage::Unmanaged);
    enqueue_pending_storage_cleanup(dir.path(), &app_a, "first").unwrap();

    let mut app_b = sample_app(InstalledSageAppStorage::Unmanaged);
    app_b.common.id = "url-other".into();
    app_b.common.name = "Other App".into();

    enqueue_pending_storage_cleanup(dir.path(), &app_b, "second").unwrap();

    let entries = read_pending_storage_cleanup_entries(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].app_id, "url-other");
    assert_eq!(entries[0].app_name, "Other App");
    assert_eq!(entries[0].attempt_count, 2);
    assert_eq!(entries[0].last_error.as_deref(), Some("second"));
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
    assert_eq!(entries[0].app_id, app.common.id);
    assert_eq!(entries[0].origin_id, app.common.origin_id);
    assert!(entries[0].cleanup_pending);
    assert!(entries[0].storage_may_contain_secrets);
}

#[test]
fn enqueue_retired_app_origin_updates_existing_origin_entry() {
    let dir = tempdir().unwrap();
    let app = sample_app(InstalledSageAppStorage::Unmanaged);

    enqueue_retired_app_origin(dir.path(), &app, true).unwrap();
    enqueue_retired_app_origin(dir.path(), &app, false).unwrap();

    let entries = read_retired_app_origins(dir.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].cleanup_pending);
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
    assert!(entries[0].storage_may_contain_secrets);
    assert!(entries[0].cleanup_pending);
}
