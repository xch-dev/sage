mod common;

use std::path::Path;

use common::{sample_installed_app, sample_manifest_file};
use sage_apps::bridge::capabilities::UserBridgeCapability;
use sage_apps::lifecycle::registry::{
    app_dir, read_installed_app_by_id, write_installed_app_metadata,
};
use sage_apps::lifecycle::update::permissions::{
    grant_requested_capability_internal, grant_requested_network_whitelist_entry_internal,
    update_app_permissions,
};
use sage_apps::lifecycle::update::types::{GrantCapabilityOutcome, GrantNetworkWhitelistOutcome};
use sage_apps::types::{
    InstalledSageAppStorage, SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot,
    SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities,
    SageRequestedNetworkPermissions, SageRequestedPermissions, UserSageApp,
};
use tempfile::tempdir;

fn entries(
    values: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
) -> Vec<SageNetworkWhitelistEntry> {
    values.into_iter().collect()
}

fn caps(values: impl IntoIterator<Item = UserBridgeCapability>) -> Vec<UserBridgeCapability> {
    values.into_iter().collect()
}

fn sample_app(base: &Path, app_id: &str) -> UserSageApp {
    let mut app = sample_installed_app(base, app_id, "Test App");

    let requested_permissions = SageRequestedPermissions::new(
        SageRequestedNetworkPermissions::new(
            [network_whitelist_entry("https", "required.example.com")],
            [network_whitelist_entry("wss", "optional.example.com")],
        ),
        SageRequestedCapabilities::new(
            [],
            [
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
        ),
    )
    .unwrap();

    let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
        name: "Test App".to_string(),
        version: "1.0.0".to_string(),
        permissions: requested_permissions.clone(),
        files: Vec::from([sample_manifest_file("index.html", 1)]),
        entry: Some("index.html".to_string()),
        icon: Some("icon.png".to_string()),
        author: None,
        donation: None,
    })
    .unwrap();

    app.common.requested_permissions = requested_permissions;
    app.common.active_snapshot = SageAppSnapshot {
        manifest_hash: "hash".to_string(),
        snapshot_dir: app.common.app_dir.clone(),
        total_bytes: 1,
        manifest,
    };
    app.common.granted_permissions =
        SageGrantedPermissions::new(&app.common.requested_permissions, [], []).unwrap();
    app.common.storage = InstalledSageAppStorage::Unmanaged;

    app
}

#[test]
fn update_app_permissions_internal_persists_required_network_entries() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let granted = SageGrantedPermissions::new(&app.common.requested_permissions, [], []).unwrap();

    let updated = update_app_permissions(dir.path(), &app.common.id, granted).unwrap();

    assert_eq!(
        entries(
            updated
                .common
                .granted_permissions
                .network()
                .whitelist()
                .cloned()
        ),
        [network_whitelist_entry("https", "required.example.com")]
    );

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        entries(
            reloaded
                .common
                .granted_permissions
                .network()
                .whitelist()
                .cloned()
        ),
        [network_whitelist_entry("https", "required.example.com")]
    );
}

#[test]
fn update_app_permissions_internal_rejects_unrequested_capability() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let err = SageGrantedPermissions::new(
        &app.common.requested_permissions,
        [UserBridgeCapability::WalletSendXchAutoSubmit],
        [],
    )
    .unwrap_err();

    let err = err.to_string();
    assert!(err.contains("not requested in manifest"));
    assert!(err.contains(UserBridgeCapability::WalletSendXchAutoSubmit.key()));
}

#[test]
fn grant_requested_capability_internal_grants_optional_capability() {
    let dir = tempdir().unwrap();
    let app = sample_app(dir.path(), "app-1");
    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome = grant_requested_capability_internal(
        dir.path(),
        &app.common.id,
        UserBridgeCapability::WalletSendXch,
    )
    .unwrap();

    match outcome {
        GrantCapabilityOutcome::Granted { capability, change } => {
            assert_eq!(capability, UserBridgeCapability::WalletSendXch);
            assert_eq!(change.added, [UserBridgeCapability::WalletSendXch]);
            assert!(change.removed.is_empty());
            assert_eq!(change.full, [UserBridgeCapability::WalletSendXch]);
        }
        GrantCapabilityOutcome::AlreadyGranted { .. } => {
            panic!("expected capability to be newly granted")
        }
    }

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        caps(reloaded.common.granted_permissions.capabilities().copied()),
        [UserBridgeCapability::WalletSendXch]
    );
}

#[test]
fn grant_requested_capability_internal_returns_already_granted_when_present() {
    let dir = tempdir().unwrap();
    let mut app = sample_app(dir.path(), "app-1");

    app.common.granted_permissions = SageGrantedPermissions::new(
        &app.common.requested_permissions,
        [UserBridgeCapability::WalletSendXch],
        [network_whitelist_entry("https", "required.example.com")],
    )
    .unwrap();

    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome = grant_requested_capability_internal(
        dir.path(),
        &app.common.id,
        UserBridgeCapability::WalletSendXch,
    )
    .unwrap();

    match outcome {
        GrantCapabilityOutcome::AlreadyGranted {
            capability,
            full_granted_capabilities,
        } => {
            assert_eq!(capability, UserBridgeCapability::WalletSendXch);
            assert_eq!(
                full_granted_capabilities,
                [UserBridgeCapability::WalletSendXch]
            );
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

    assert!(
        err.to_string()
            .contains("Capability was not requested by app manifest")
    );
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
        &network_whitelist_entry("WSS", "OPTIONAL.EXAMPLE.COM"),
    )
    .unwrap();

    match outcome {
        GrantNetworkWhitelistOutcome::Granted { entry, change } => {
            assert_eq!(
                entry,
                network_whitelist_entry("wss", "optional.example.com")
            );
            assert_eq!(
                change.added,
                [
                    network_whitelist_entry("https", "required.example.com"),
                    network_whitelist_entry("wss", "optional.example.com"),
                ]
            );
            assert!(change.removed.is_empty());
            assert_eq!(
                change.full,
                [
                    network_whitelist_entry("https", "required.example.com"),
                    network_whitelist_entry("wss", "optional.example.com"),
                ]
            );
        }
        GrantNetworkWhitelistOutcome::AlreadyGranted { .. } => {
            panic!("expected network entry to be newly granted")
        }
    }

    let reloaded = read_installed_app_by_id(dir.path(), &app.common.id).unwrap();
    assert_eq!(
        entries(
            reloaded
                .common
                .granted_permissions
                .network()
                .whitelist()
                .cloned()
        ),
        [
            network_whitelist_entry("https", "required.example.com"),
            network_whitelist_entry("wss", "optional.example.com"),
        ]
    );
}

#[test]
fn grant_requested_network_whitelist_entry_internal_returns_already_granted_when_present() {
    let dir = tempdir().unwrap();
    let mut app = sample_app(dir.path(), "app-1");

    app.common.granted_permissions = SageGrantedPermissions::new(
        &app.common.requested_permissions,
        [],
        [network_whitelist_entry("https", "required.example.com")],
    )
    .unwrap();

    let app_path = app_dir(dir.path(), &app.common.id);
    write_installed_app_metadata(&app, &app_path).unwrap();

    let outcome = grant_requested_network_whitelist_entry_internal(
        dir.path(),
        &app.common.id,
        &network_whitelist_entry("https", "required.example.com"),
    )
    .unwrap();

    match outcome {
        GrantNetworkWhitelistOutcome::AlreadyGranted {
            entry,
            full_granted_network_whitelist,
        } => {
            assert_eq!(
                entry,
                network_whitelist_entry("https", "required.example.com")
            );
            assert_eq!(
                full_granted_network_whitelist,
                [network_whitelist_entry("https", "required.example.com")]
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
        &network_whitelist_entry("https", "evil.example.com"),
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("Network whitelist entry was not requested by app manifest")
    );
}

fn network_whitelist_entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
    SageNetworkWhitelistEntry::new(scheme, host).unwrap()
}
