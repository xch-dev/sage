use crate::bridge::USER_BRIDGE_CHANNEL;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::event_emit::emit_bridge_event_to_app_id;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::lifecycle::update::types::{
    AppUpdateResult, GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedPermissionsChange,
};
use crate::lifecycle::{read_installed_app_by_id, write_installed_app_metadata};
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry, UserSageApp};
use std::path::Path;
use tauri::AppHandle;

pub async fn update_app_permissions(
    app: &AppHandle,
    base_path: &Path,
    app_id: &str,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<UserSageApp> {
    let update_result = update_app_permissions_internal(base_path, app_id, granted_permissions)?;

    emit_granted_permissions_change(app, app_id, update_result.change()).await;

    Ok(update_result.into_app())
}

pub async fn grant_capability(
    app: &AppHandle,
    base_path: &Path,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<GrantCapabilityOutcome> {
    let update = grant_capability_internal(base_path, app_id, capability)?;

    emit_granted_permissions_change(app, app_id, update.change()).await;

    Ok(GrantCapabilityOutcome::from_update(capability, &update))
}

pub async fn grant_network_whitelist_entry(
    app_handle: &AppHandle,
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let update = grant_network_whitelist_entry_internal(base_path, app_id, entry)?;

    emit_granted_permissions_change(app_handle, app_id, update.change()).await;

    Ok(GrantNetworkWhitelistOutcome::from_update(entry, &update))
}

fn update_app_permissions_internal(
    base_path: &Path,
    app_id: &str,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<AppUpdateResult> {
    let mut app = read_installed_app_by_id(base_path, app_id)?;

    let previous_permissions = app.common().granted_permissions().clone();

    app.common_mut().update_permissions(granted_permissions)?;

    write_installed_app_metadata(&app)?;

    let change =
        GrantedPermissionsChange::diff(&previous_permissions, app.common().granted_permissions());

    Ok(AppUpdateResult::new(app, change))
}

fn grant_capability_internal(
    base_path: &Path,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<AppUpdateResult> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let granted_permissions = app
        .common()
        .granted_permissions()
        .with_capability_added(app.common().requested_permissions(), capability)?;

    update_app_permissions_internal(base_path, app_id, &granted_permissions)
}

fn grant_network_whitelist_entry_internal(
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<AppUpdateResult> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let granted_permissions = app
        .common()
        .granted_permissions()
        .with_network_whitelist_entry_added(app.common().requested_permissions(), entry.clone())?;

    update_app_permissions_internal(base_path, app_id, &granted_permissions)
}

async fn emit_granted_permissions_change(
    app: &AppHandle,
    app_id: &str,
    change: &GrantedPermissionsChange,
) {
    let capability_change = change.capabilities();
    if !capability_change.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            app,
            app_id,
            EventForApp::from_capabilities_change(USER_BRIDGE_CHANNEL, capability_change),
        )
        .await;
    }
    let network_change = change.network_whitelist();

    if !network_change.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            app,
            app_id,
            EventForApp::from_network_whitelist_change(USER_BRIDGE_CHANNEL, network_change),
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::registry::{
        app_dir, read_installed_app_by_id, write_installed_app_metadata,
    };
    use crate::types::{
        InstalledSageAppStorage, SageAppCommon, SageAppIdentity, SageAppManifestFile,
        SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot,
        SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities,
        SageRequestedNetworkPermissions, SageRequestedPermissions, UserSageApp, UserSageAppSource,
    };
    use std::path::Path;
    use tempfile::tempdir;

    fn network_whitelist_entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn entries(
        values: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Vec<SageNetworkWhitelistEntry> {
        values.into_iter().collect()
    }

    fn caps(values: impl IntoIterator<Item = UserBridgeCapability>) -> Vec<UserBridgeCapability> {
        values.into_iter().collect()
    }

    fn write_file(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
    }

    fn sample_app(base: &Path, app_id: &str) -> UserSageApp {
        let app_dir = app_dir(base, app_id);
        std::fs::create_dir_all(&app_dir).unwrap();
        write_file(&app_dir.join("index.html"), "test");

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
            files: Vec::from([SageAppManifestFile::new("index.html", "a".repeat(64), 4).unwrap()]),
            entry: Some("index.html".to_string()),
            icon: None,
            author: None,
            donation: None,
        })
        .unwrap();

        let granted_permissions =
            SageGrantedPermissions::new(&requested_permissions, [], []).unwrap();

        let snapshot =
            SageAppSnapshot::new("hash", app_dir.to_string_lossy().to_string(), manifest).unwrap();

        let common = SageAppCommon::new(
            SageAppIdentity::new(app_id, app_id, app_dir.to_string_lossy().to_string()).unwrap(),
            granted_permissions,
            InstalledSageAppStorage::Unmanaged,
            snapshot,
        )
        .unwrap();

        UserSageApp::new_installed(
            common,
            UserSageAppSource::url("https://example.com/app/").unwrap(),
        )
    }

    #[test]
    fn update_app_permissions_persists_required_network_entries() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        write_installed_app_metadata(&app).unwrap();

        let granted =
            SageGrantedPermissions::new(app.common().requested_permissions(), [], []).unwrap();

        let update_result =
            update_app_permissions_internal(dir.path(), app.common().id(), &granted).unwrap();

        assert_eq!(
            entries(
                update_result
                    .app()
                    .common()
                    .granted_permissions()
                    .network()
                    .whitelist_iter()
                    .cloned()
            ),
            [network_whitelist_entry("https", "required.example.com")]
        );

        let reloaded = read_installed_app_by_id(dir.path(), app.common().id()).unwrap();
        assert_eq!(
            entries(
                reloaded
                    .common()
                    .granted_permissions()
                    .network()
                    .whitelist_iter()
                    .cloned()
            ),
            [network_whitelist_entry("https", "required.example.com")]
        );
    }

    #[test]
    fn grant_requested_capability_grants_optional_capability() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        write_installed_app_metadata(&app).unwrap();

        let update_result = grant_capability_internal(
            dir.path(),
            app.common().id(),
            UserBridgeCapability::WalletSendXch,
        )
        .unwrap();
        let outcome = GrantCapabilityOutcome::from_update(
            UserBridgeCapability::WalletSendXch,
            &update_result,
        );

        match outcome {
            GrantCapabilityOutcome::Granted { capability, change } => {
                assert_eq!(capability, UserBridgeCapability::WalletSendXch);
                assert_eq!(change.added, [UserBridgeCapability::WalletSendXch]);
                assert!(change.removed.is_empty());
                assert_eq!(change.full, [UserBridgeCapability::WalletSendXch]);
            }
            GrantCapabilityOutcome::AlreadyGranted { .. } => {
                panic!("expected capability to be newly granted");
            }
        }

        let reloaded = read_installed_app_by_id(dir.path(), app.common().id()).unwrap();
        assert_eq!(
            caps(
                reloaded
                    .common()
                    .granted_permissions()
                    .capabilities()
                    .copied()
            ),
            [UserBridgeCapability::WalletSendXch]
        );
    }

    #[test]
    fn grant_requested_capability_returns_already_granted_when_present() {
        let dir = tempdir().unwrap();
        let mut app = sample_app(dir.path(), "app-1");

        let granted_permissions = SageGrantedPermissions::new(
            app.common().requested_permissions(),
            [UserBridgeCapability::WalletSendXch],
            [network_whitelist_entry("https", "required.example.com")],
        )
        .unwrap();

        app.common_mut()
            .update_permissions(&granted_permissions)
            .unwrap();

        write_installed_app_metadata(&app).unwrap();

        let update_result = grant_capability_internal(
            dir.path(),
            app.common().id(),
            UserBridgeCapability::WalletSendXch,
        )
        .unwrap();
        let outcome = GrantCapabilityOutcome::from_update(
            UserBridgeCapability::WalletSendXch,
            &update_result,
        );

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
                panic!("expected already-granted outcome");
            }
        }
    }

    #[test]
    fn grant_requested_network_whitelist_entry_grants_optional_entry() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        write_installed_app_metadata(&app).unwrap();

        let entry = network_whitelist_entry("WSS", "OPTIONAL.EXAMPLE.COM");
        let update_result =
            grant_network_whitelist_entry_internal(dir.path(), app.common().id(), &entry).unwrap();
        let outcome = GrantNetworkWhitelistOutcome::from_update(&entry, &update_result);

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
                panic!("expected network entry to be newly granted");
            }
        }

        let reloaded = read_installed_app_by_id(dir.path(), app.common().id()).unwrap();
        assert_eq!(
            entries(
                reloaded
                    .common()
                    .granted_permissions()
                    .network()
                    .whitelist_iter()
                    .cloned()
            ),
            [
                network_whitelist_entry("https", "required.example.com"),
                network_whitelist_entry("wss", "optional.example.com"),
            ]
        );
    }

    #[test]
    fn grant_requested_network_whitelist_entry_returns_already_granted_when_present() {
        let dir = tempdir().unwrap();
        let mut app = sample_app(dir.path(), "app-1");

        let granted_permissions = SageGrantedPermissions::new(
            app.common().requested_permissions(),
            [],
            [network_whitelist_entry("https", "required.example.com")],
        )
        .unwrap();

        app.common_mut()
            .update_permissions(&granted_permissions)
            .unwrap();

        write_installed_app_metadata(&app).unwrap();

        let entry = network_whitelist_entry("https", "required.example.com");
        let update_result =
            grant_network_whitelist_entry_internal(dir.path(), app.common().id(), &entry).unwrap();
        let outcome = GrantNetworkWhitelistOutcome::from_update(&entry, &update_result);

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
                panic!("expected already-granted outcome");
            }
        }
    }
}
