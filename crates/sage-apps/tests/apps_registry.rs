mod common;

use std::fs;

use common::{sample_installed_app};
use sage_apps::lifecycle::registry::{
    app_dir, apps_root, list_installed_apps_internal, parse_network_permission_target,
    read_installed_app_by_id, read_installed_user_app_by_origin_id,
    read_pending_storage_cleanup_entries, read_retired_app_origins,
    write_installed_app_metadata, write_pending_storage_cleanup_entries,
    write_retired_app_origins,
};
use sage_apps::types::{ListedSageApp, PendingStorageCleanupEntry, PendingStorageCleanupTarget, RetiredAppOriginEntry, SageAppPackageManifest, SageAppPackageManifestParts, SageNetworkWhitelistEntry, SageRequestedPermissions, UserSageAppPendingUpdate};
use tempfile::tempdir;
use crate::common::{sample_manifest_file};

#[test]
fn installed_app_metadata_roundtrips() {
    let base = tempdir().unwrap();
    let app_id = "app-1";
    let dir = app_dir(base.path(), app_id);

    let mut app = sample_installed_app(base.path(), app_id, "Alpha");
    app.common.app_dir = dir.to_string_lossy().to_string();

    write_installed_app_metadata(&app, &dir).unwrap();
    let loaded = read_installed_app_by_id(base.path(), app_id).unwrap();

    assert_eq!(loaded.common.id, app.common.id);
    assert_eq!(loaded.common.name, app.common.name);
    assert_eq!(loaded.common.granted_permissions, app.common.granted_permissions);
    assert_eq!(loaded.common.capability_flags.has_external_access, false);
}

fn without_system_apps(listed: Vec<ListedSageApp>) -> Vec<ListedSageApp> {
    listed
        .into_iter()
        .filter(|entry| !matches!(entry, ListedSageApp::System(_)))
        .collect()
}

#[test]
fn installed_app_metadata_roundtrips_pending_update() {
    let base = tempdir().unwrap();
    let app_id = "app-1";
    let dir = app_dir(base.path(), app_id);

    let mut app = sample_installed_app(base.path(), app_id, "Alpha");
    app.common.app_dir = dir.to_string_lossy().to_string();
    app.pending_update = Some(UserSageAppPendingUpdate {
        app_url: "https://example.com/app/".to_string(),
        manifest_url: "https://example.com/app/sage-manifest.json".to_string(),
        manifest_hash: "pending-hash".to_string(),
        manifest: sample_manifest("Alpha Updated"),
    });

    write_installed_app_metadata(&app, &dir).unwrap();
    let loaded = read_installed_app_by_id(base.path(), app_id).unwrap();

    let pending = loaded.pending_update.expect("pending update should survive roundtrip");
    assert_eq!(pending.manifest_hash, "pending-hash");
    assert_eq!(pending.manifest.name(), "Alpha Updated");
}

#[test]
fn corrupted_metadata_is_reported_as_corrupted_listing() {
    let base = tempdir().unwrap();
    let dir = app_dir(base.path(), "broken-app");
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".sage-installed.json"), "{ definitely not json").unwrap();

    let listed = without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
    assert_eq!(listed.len(), 1);

    match &listed[0] {
        ListedSageApp::Corrupted(app) => {
            assert_eq!(app.id, "broken-app");
            assert!(!app.error.is_empty());
        }
        ListedSageApp::User(_) | ListedSageApp::System(_) => {
            panic!("expected corrupted app listing")
        }
    }
}

#[test]
fn corrupted_persisted_network_entry_is_reported_as_corrupted_listing() {
    let base = tempdir().unwrap();
    let dir = app_dir(base.path(), "broken-app");
    fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join(".sage-installed.json"),
        r#"{
  "id": "broken-app",
  "originId": "broken-app",
  "name": "Broken App",
  "version": "1.0.0",
  "appDir": "/tmp/broken-app",
  "entryFile": "index.html",
  "iconFile": "icon.png",
  "requestedPermissions": {
    "network": {
      "whitelist": {
        "required": ["https://ok.example.com/path"],
        "optional": []
      }
    },
    "capabilities": {
      "required": [],
      "optional": []
    }
  },
  "grantedPermissions": {
    "capabilities": [],
    "network": {
      "whitelist": []
    }
  },
  "capabilityFlags": {
    "hasSecretAccess": false,
    "hasExternalAccess": false,
    "storageMayContainSecrets": false,
    "isolated": false
  },
  "storage": {
    "kind": "unmanaged"
  },
  "source": {
    "kind": "zip"
  },
  "activeSnapshot": {
    "manifestHash": "hash",
    "snapshotDir": "/tmp/snapshot",
    "totalBytes": 1,
    "manifest": {
      "name": "Broken App",
      "version": "1.0.0",
      "permissions": {
        "network": {
          "whitelist": {
            "required": [],
            "optional": []
          }
        },
        "capabilities": {
          "required": [],
          "optional": []
        }
      },
      "files": [
        {
          "path": "index.html",
          "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "size": 1
        }
      ],
      "entry": "index.html",
      "icon": "icon.png"
    }
  },
  "pendingUpdate": null
}"#,
    )
        .unwrap();

    let listed = without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
    assert_eq!(listed.len(), 1);

    match &listed[0] {
        ListedSageApp::Corrupted(app) => {
            assert_eq!(app.id, "broken-app");
            assert!(
                app.error.contains("network entry")
                    || app.error.contains("invalid host")
                    || app.error.contains("failed to parse installed app metadata")
                    || app.error.contains("failed to parse persisted required network entry"),
                "unexpected error: {}",
                app.error
            );
        }
        ListedSageApp::User(_) | ListedSageApp::System(_) => {
            panic!("expected corrupted app listing")
        }
    }
}

#[test]
fn installed_apps_are_sorted_by_name() {
    let base = tempdir().unwrap();

    let alpha_dir = app_dir(base.path(), "a");
    let mut alpha = sample_installed_app(base.path(), "a", "Alpha");
    alpha.common.app_dir = alpha_dir.to_string_lossy().to_string();
    write_installed_app_metadata(&alpha, &alpha_dir).unwrap();

    let zeta_dir = app_dir(base.path(), "z");
    let mut zeta = sample_installed_app(base.path(), "z", "Zeta");
    zeta.common.app_dir = zeta_dir.to_string_lossy().to_string();
    write_installed_app_metadata(&zeta, &zeta_dir).unwrap();

    let listed = without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
    let names: Vec<_> = listed
        .into_iter()
        .map(|entry| match entry {
            ListedSageApp::User(app) => app.common.name.to_string(),
            ListedSageApp::System(app) => app.common.name.to_string(),
            ListedSageApp::Corrupted(app) => app.id,
        })
        .collect();

    assert_eq!(names, vec!["Alpha".to_string(), "Zeta".to_string()]);
}

#[test]
fn list_installed_apps_ignores_tmp_directories() {
    let base = tempdir().unwrap();
    let root = apps_root(base.path());
    fs::create_dir_all(root.join(".tmp-123")).unwrap();

    let listed = without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
    assert!(listed.is_empty());
}

#[test]
fn list_installed_apps_ignores_directories_without_metadata() {
    let base = tempdir().unwrap();
    let root = apps_root(base.path());
    fs::create_dir_all(root.join("missing-metadata")).unwrap();

    let listed = without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
    assert!(listed.is_empty());
}

#[test]
fn parse_network_permission_target_normalizes_case() {
    let parsed = parse_network_permission_target("HTTPS://Example.COM").unwrap();
    assert_eq!(
        parsed,
        SageNetworkWhitelistEntry::new("https", "example.com").unwrap()
    );
}

#[test]
fn parse_network_permission_target_rejects_missing_scheme_separator() {
    let err = parse_network_permission_target("example.com").unwrap_err();
    assert!(err.contains("missing scheme"));
}

#[test]
fn parse_network_permission_target_rejects_unsupported_scheme() {
    let err = parse_network_permission_target("http://example.com").unwrap_err();
    assert!(err.contains("only https and wss allowed"));
}

#[test]
fn parse_network_permission_target_rejects_invalid_host_chars() {
    assert!(parse_network_permission_target("https://example.com/path").is_err());
    assert!(parse_network_permission_target("https://example.com?x=1").is_err());
    assert!(parse_network_permission_target("https://example.com#frag").is_err());
    assert!(parse_network_permission_target("https://exa mple.com").is_err());
}

#[test]
fn read_installed_app_by_origin_id_finds_matching_app() {
    let base = tempdir().unwrap();

    let alpha_dir = app_dir(base.path(), "a");
    let mut alpha = sample_installed_app(base.path(), "a", "Alpha");
    alpha.common.origin_id = "origin-a".to_string();
    alpha.common.app_dir = alpha_dir.to_string_lossy().to_string();
    write_installed_app_metadata(&alpha, &alpha_dir).unwrap();

    let beta_dir = app_dir(base.path(), "b");
    let mut beta = sample_installed_app(base.path(), "b", "Beta");
    beta.common.origin_id = "origin-b".to_string();
    beta.common.app_dir = beta_dir.to_string_lossy().to_string();
    write_installed_app_metadata(&beta, &beta_dir).unwrap();

    let found = read_installed_user_app_by_origin_id(base.path(), "origin-b").unwrap();
    assert_eq!(found.common.id, "b");
}

#[test]
fn read_installed_app_by_origin_id_errors_when_missing() {
    let base = tempdir().unwrap();
    let err = read_installed_user_app_by_origin_id(base.path(), "missing").unwrap_err();
    assert!(err
        .to_string()
        .contains("no installed app found for origin id"));
}

#[test]
fn pending_storage_cleanup_entries_roundtrip() {
    let base = tempdir().unwrap();

    let entries = vec![PendingStorageCleanupEntry {
        id: "cleanup-1".to_string(),
        app_id: "app-1".to_string(),
        app_name: "App".to_string(),
        target: PendingStorageCleanupTarget::Unmanaged,
        created_at_ms: 1,
        last_attempt_at_ms: Some(2),
        attempt_count: 3,
        last_error: Some("boom".to_string()),
    }];

    write_pending_storage_cleanup_entries(base.path(), &entries).unwrap();
    let loaded = read_pending_storage_cleanup_entries(base.path()).unwrap();
    assert_eq!(loaded, entries);
}

#[test]
fn retired_app_origins_roundtrip() {
    let base = tempdir().unwrap();

    let entries = vec![RetiredAppOriginEntry {
        id: "retired-1".to_string(),
        app_id: "app-1".to_string(),
        app_name: "App".to_string(),
        origin_id: "origin-1".to_string(),
        created_at_ms: 1,
        storage_may_contain_secrets: true,
        cleanup_pending: true,
    }];

    write_retired_app_origins(base.path(), &entries).unwrap();
    let loaded = read_retired_app_origins(base.path()).unwrap();
    assert_eq!(loaded, entries);
}

fn sample_manifest(name: &str) -> SageAppPackageManifest {
    SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        permissions: SageRequestedPermissions::empty(),
        files: vec![sample_manifest_file("index.html", 1)],
        entry: Some("index.html".to_string()),
        icon: Some("icon.png".to_string()),
        author: None,
        donation: None,
    }).unwrap()
}
