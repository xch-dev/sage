use std::fs;
use std::path::Path;
use sage_apps::lifecycle::app_dir;
use sage_apps::types::{InstalledSageAppStorage, SageAppCommon, SageAppManifestFile, SageAppPackageManifest, SageAppSnapshot, SageGrantedPermissions, SageRequestedPermissions, UserSageApp, UserSageAppSource, SageAppPackageManifestParts};

pub fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
    SageAppManifestFile {
        path: path.to_string(),
        sha256: "a".repeat(64),
        size,
    }
}

pub fn sample_installed_app(base: &Path, app_id: &str, name: &str) -> UserSageApp {
    let app_dir = app_dir(base, app_id);
    fs::create_dir_all(&app_dir).unwrap();

    let requested_permissions = SageRequestedPermissions::empty();

    let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        permissions: requested_permissions.clone(),
        files: vec![SageAppManifestFile {
            path: "index.html".into(),
            sha256: "a".repeat(64),
            size: 1,
        }],
        entry: Some("index.html".into()),
        icon: Some("icon.png".into()),
        author: None,
        donation: None,
    })
        .unwrap();

    let snapshot = SageAppSnapshot {
        manifest_hash: "hash".into(),
        snapshot_dir: app_dir.to_string_lossy().to_string(),
        total_bytes: 1,
        manifest: manifest.clone(),
    };

    let granted_permissions =
        SageGrantedPermissions::new(&requested_permissions, [], []).unwrap();

    let common = SageAppCommon::new(
        app_id.into(),
        app_id.into(),
        app_dir.to_string_lossy().to_string(),
        &manifest,
        granted_permissions,
        InstalledSageAppStorage::Unmanaged,
        snapshot,
    )
        .unwrap();

    UserSageApp {
        common,
        source: UserSageAppSource::Url {
            app_url: "https://example.com/app/".into(),
            manifest_url: "https://example.com/app/sage-manifest.json".into(),
        },
        pending_update: None,
    }
}