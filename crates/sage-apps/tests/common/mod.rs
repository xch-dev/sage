use std::fs;
use std::path::Path;
use sage_apps::lifecycle::app_dir;
use sage_apps::types::{InstalledSageAppStorage, SageAppFlags, SageAppCommon, SageAppManifestFile, SageAppPackageManifest, SageAppSnapshot, SageGrantedNetworkPermissions, SageGrantedPermissions, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions, UserSageApp, UserSageAppSource};

pub fn empty_permissions() -> SageRequestedPermissions {
    SageRequestedPermissions {
        network: SageRequestedNetworkPermissions {
            whitelist: SageRequestedNetworkWhitelist {
                required: vec![],
                optional: vec![],
            },
        },
        capabilities: SageRequestedCapabilities {
            required: vec![],
            optional: vec![],
        },
    }
}

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

    UserSageApp {
        common: SageAppCommon {
            id: app_id.into(),
            origin_id: app_id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            app_dir: app_dir.to_string_lossy().to_string(),
            entry_file: "index.html".into(),
            icon_file: "icon.png".into(),
            requested_permissions: SageRequestedPermissions::default(),
            granted_permissions: SageGrantedPermissions {
                capabilities: vec![],
                network: SageGrantedNetworkPermissions { whitelist: vec![] },
            },
            capability_flags: SageAppFlags::default(),
            storage: InstalledSageAppStorage::Unmanaged,
            active_snapshot: SageAppSnapshot {
                manifest_hash: "hash".into(),
                snapshot_dir: app_dir.to_string_lossy().to_string(),
                total_bytes: 1,
                manifest: SageAppPackageManifest {
                    name: name.into(),
                    version: "1.0.0".into(),
                    permissions: SageRequestedPermissions::default(),
                    files: vec![SageAppManifestFile {
                        path: "index.html".into(),
                        sha256: "a".repeat(64),
                        size: 1,
                    }],
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
