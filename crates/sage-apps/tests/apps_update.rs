mod common;

use std::path::Path;

use common::{sample_installed_app, sample_manifest_file};
use sage_apps::lifecycle::registry::{
    app_dir, read_installed_app_by_id, write_installed_app_metadata,
};
use sage_apps::types::{
    InstalledSageAppStorage, SageAppFlags, SageAppPackageManifest,
    SageAppSnapshot, SageGrantedNetworkPermissions, SageGrantedPermissions,
    SageNetworkPermissionTarget, SageRequestedCapabilities,
    SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist,
    SageRequestedPermissions, UserSageApp,
};
use tempfile::tempdir;
use sage_apps::bridge::capabilities::UserBridgeCapability;
use sage_apps::lifecycle::update::permissions::{grant_requested_capability_internal, grant_requested_network_whitelist_entry_internal, update_app_permissions};
use sage_apps::lifecycle::update::types::{GrantCapabilityOutcome, GrantNetworkWhitelistOutcome};

fn sample_app(base: &Path, app_id: &str) -> UserSageApp {
    let mut app = sample_installed_app(base, app_id, "Test App");

    app.common.requested_permissions = SageRequestedPermissions {
        network: SageRequestedNetworkPermissions {
            whitelist: SageRequestedNetworkWhitelist {
                required: vec![SageNetworkPermissionTarget {
                    scheme: "https".to_string(),
                    host: "required.example.com".to_string(),
                }],
                optional: vec![SageNetworkPermissionTarget {
                    scheme: "wss".to_string(),
                    host: "optional.example.com".to_string(),
                }],
            },
        },
        capabilities: SageRequestedCapabilities {
            required: vec![],
            optional: vec![
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
        },
    };

    app.common.active_snapshot = SageAppSnapshot {
        manifest_hash: "hash".to_string(),
        snapshot_dir: app.common.app_dir.clone(),
        total_bytes: 1,
        manifest: SageAppPackageManifest {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: app.common.requested_permissions.clone(),
            files: vec![sample_manifest_file("index.html", 1)],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        },
    };

    app.common.storage = InstalledSageAppStorage::Unmanaged;
    app
}

#[test]
fn update_app_permissions_internal_persists_required_network_entries() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let updated = update_app_permissions(
        dir.path(),
        &app.common.id,
        SageGrantedPermissions {
            capabilities: vec![],
            network: SageGrantedNetworkPermissions { whitelist: vec![] },
        },
        false,
    )
        .unwrap();

    assert_eq!(
        updated.common.granted_permissions.network.whitelist,
        vec![SageNetworkPermissionTarget {
            scheme: "https".to_string(),
            host: "required.example.com".to_string(),
        }]
    );

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        reloaded.common.granted_permissions.network.whitelist,
        vec![SageNetworkPermissionTarget {
            scheme: "https".to_string(),
            host: "required.example.com".to_string(),
        }]
    );
}

#[test]
fn update_app_permissions_internal_rejects_unrequested_capability() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let err = update_app_permissions(
        dir.path(),
        &app.common.id,
        SageGrantedPermissions {
            capabilities: vec![UserBridgeCapability::WalletSendXchAutoSubmit],
            network: SageGrantedNetworkPermissions { whitelist: vec![] },
        },
        false,
    )
        .unwrap_err();

    let err = err.to_string();
    assert!(err.contains("not requested in manifest"));
    assert!(err.contains(UserBridgeCapability::WalletSendXchAutoSubmit.key()));
}

#[test]
fn update_app_permissions_internal_can_clear_storage_taint_without_capabilities() {
    let dir = tempdir().unwrap();
    let mut app = sample_app(dir.path(), "app-1");
    app.common.capability_flags = SageAppFlags {
        has_secret_access: false,
        has_external_access: false,
        storage_may_contain_secrets: true,
        isolated: true,
    };

    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let updated = update_app_permissions(
        dir.path(),
        &app.common.id,
        SageGrantedPermissions {
            capabilities: vec![],
            network: SageGrantedNetworkPermissions { whitelist: vec![] },
        },
        true,
    )
        .unwrap();

    assert!(!updated.common.capability_flags.storage_may_contain_secrets);
    assert!(!updated.common.capability_flags.isolated);
}

#[test]
fn grant_requested_capability_internal_grants_optional_capability() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome =
        grant_requested_capability_internal(dir.path(), &app.common.id, UserBridgeCapability::WalletSendXch)
            .unwrap();

    match outcome {
        GrantCapabilityOutcome::Granted { capability, change } => {
            assert_eq!(capability, UserBridgeCapability::WalletSendXch);
            assert_eq!(change.added, vec![UserBridgeCapability::WalletSendXch]);
            assert!(change.removed.is_empty());
            assert_eq!(change.full, vec![UserBridgeCapability::WalletSendXch]);
        }
        GrantCapabilityOutcome::AlreadyGranted { .. } => {
            panic!("expected capability to be newly granted")
        }
    }

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        reloaded.common.granted_permissions.capabilities,
        vec![UserBridgeCapability::WalletSendXch]
    );
}

#[test]
fn grant_requested_capability_internal_returns_already_granted_when_present() {
    let dir = tempdir().unwrap();
    let mut app = sample_app(dir.path(), "app-1");
    app.common.granted_permissions.capabilities = vec![UserBridgeCapability::WalletSendXch];

    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome =
        grant_requested_capability_internal(dir.path(), &app.common.id, UserBridgeCapability::WalletSendXch)
            .unwrap();

    match outcome {
        GrantCapabilityOutcome::AlreadyGranted {
            capability,
            full_granted_capabilities,
        } => {
            assert_eq!(capability, UserBridgeCapability::WalletSendXch);
            assert_eq!(full_granted_capabilities, vec![UserBridgeCapability::WalletSendXch]);
        }
        GrantCapabilityOutcome::Granted { .. } => {
            panic!("expected already-granted outcome")
        }
    }
}

#[test]
fn grant_requested_capability_internal_rejects_unrequested_capability() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let err = grant_requested_capability_internal(
        dir.path(),
        &app.common.id,
        UserBridgeCapability::WalletSendXchAutoSubmit,
    )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("Capability was not requested by app manifest"));
}

#[test]
fn grant_requested_network_whitelist_entry_internal_grants_optional_entry() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome = grant_requested_network_whitelist_entry_internal(
        dir.path(),
        &app.common.id,
        &SageNetworkPermissionTarget {
            scheme: "WSS".to_string(),
            host: "OPTIONAL.EXAMPLE.COM".to_string(),
        },
    )
        .unwrap();

    match outcome {
        GrantNetworkWhitelistOutcome::Granted { entry, change } => {
            assert_eq!(
                entry,
                SageNetworkPermissionTarget {
                    scheme: "wss".to_string(),
                    host: "optional.example.com".to_string(),
                }
            );
            assert_eq!(
                change.added,
                vec![
                    SageNetworkPermissionTarget {
                        scheme: "https".to_string(),
                        host: "required.example.com".to_string(),
                    },
                    SageNetworkPermissionTarget {
                        scheme: "wss".to_string(),
                        host: "optional.example.com".to_string(),
                    },
                ]
            );
            assert!(change.removed.is_empty());
            assert_eq!(
                change.full,
                vec![
                    SageNetworkPermissionTarget {
                        scheme: "https".to_string(),
                        host: "required.example.com".to_string(),
                    },
                    SageNetworkPermissionTarget {
                        scheme: "wss".to_string(),
                        host: "optional.example.com".to_string(),
                    },
                ]
            );
        }
        GrantNetworkWhitelistOutcome::AlreadyGranted { .. } => {
            panic!("expected network entry to be newly granted")
        }
    }

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        reloaded.common.granted_permissions.network.whitelist,
        vec![
            SageNetworkPermissionTarget {
                scheme: "https".to_string(),
                host: "required.example.com".to_string(),
            },
            SageNetworkPermissionTarget {
                scheme: "wss".to_string(),
                host: "optional.example.com".to_string(),
            },
        ]
    );
}

#[test]
fn grant_requested_network_whitelist_entry_internal_returns_already_granted_when_present() {
    let dir = tempdir().unwrap();
    let mut app = sample_app(dir.path(), "app-1");
    app.common.granted_permissions.network.whitelist = vec![SageNetworkPermissionTarget {
        scheme: "https".to_string(),
        host: "required.example.com".to_string(),
    }];

    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome = grant_requested_network_whitelist_entry_internal(
        dir.path(),
        &app.common.id,
        &SageNetworkPermissionTarget {
            scheme: "https".to_string(),
            host: "required.example.com".to_string(),
        },
    )
        .unwrap();

    match outcome {
        GrantNetworkWhitelistOutcome::AlreadyGranted {
            entry,
            full_granted_network_whitelist,
        } => {
            assert_eq!(
                entry,
                SageNetworkPermissionTarget {
                    scheme: "https".to_string(),
                    host: "required.example.com".to_string(),
                }
            );
            assert_eq!(
                full_granted_network_whitelist,
                vec![SageNetworkPermissionTarget {
                    scheme: "https".to_string(),
                    host: "required.example.com".to_string(),
                }]
            );
        }
        GrantNetworkWhitelistOutcome::Granted { .. } => {
            panic!("expected already-granted outcome")
        }
    }
}

#[test]
fn grant_requested_network_whitelist_entry_internal_rejects_unrequested_entry() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let err = grant_requested_network_whitelist_entry_internal(
        dir.path(),
        &app.common.id,
        &SageNetworkPermissionTarget {
            scheme: "https".to_string(),
            host: "evil.example.com".to_string(),
        },
    )
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("Network whitelist entry was not requested by app manifest"));
}
