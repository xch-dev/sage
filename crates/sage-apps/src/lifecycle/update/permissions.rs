use std::path::Path;

use anyhow::Context;
use tauri::AppHandle;

use crate::capabilities::list::UserBridgeCapability;
use crate::bridge::{emit_runtime_event_to_app_id};
use crate::bridge::methods::user::app::{GrantedCapabilitiesChangeEvent, GrantedNetworkWhitelistChangeEvent};
use crate::lifecycle::update::types::{
    AppUpdateResult, GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedPermissionsChange,
};
use crate::lifecycle::write_installed_app_metadata;
use crate::runtime::resolve_app;
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry, SharedSageApp};

pub async fn update_app_permissions_for_app(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<()> {
    let app_id = app.id();

    let update_result = apply_granted_permissions_to_app(app, granted_permissions)?;

    emit_granted_permissions_change(app_handle, &app_id, update_result.change()).await;

    Ok(())
}

pub async fn grant_capability(
    app_handle: &AppHandle,
    _base_path: &Path,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<GrantCapabilityOutcome> {
    let update = grant_capability_internal(app_handle, app_id, capability).await?;

    emit_granted_permissions_change(app_handle, app_id, update.change()).await;

    Ok(GrantCapabilityOutcome::from_update(capability, &update))
}

pub async fn grant_network_whitelist_entry(
    app_handle: &AppHandle,
    _base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let update = grant_network_whitelist_entry_internal(app_handle, app_id, entry).await?;

    emit_granted_permissions_change(app_handle, app_id, update.change()).await;

    Ok(GrantNetworkWhitelistOutcome::from_update(entry, &update))
}

async fn grant_capability_internal(
    app_handle: &AppHandle,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<AppUpdateResult> {
    let app = resolve_app_for_permission_update(app_handle, app_id).await?;

    let granted_permissions = app.try_with(|sage_app| {
        sage_app
            .common()
            .granted_permissions()
            .with_capability_added(sage_app.common().requested_permissions(), capability)
    })?;

    apply_granted_permissions_to_app(&app, &granted_permissions)
}

async fn grant_network_whitelist_entry_internal(
    app_handle: &AppHandle,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<AppUpdateResult> {
    let app = resolve_app_for_permission_update(app_handle, app_id).await?;

    let granted_permissions = app.try_with(|sage_app| {
        sage_app
            .common()
            .granted_permissions()
            .with_network_whitelist_entry_added(
                sage_app.common().requested_permissions(),
                entry.clone(),
            )
    })?;

    apply_granted_permissions_to_app(&app, &granted_permissions)
}

async fn resolve_app_for_permission_update(
    app_handle: &AppHandle,
    app_id: &str,
) -> anyhow::Result<SharedSageApp> {
    let resolved_app = resolve_app(app_handle, app_id)
        .await
        .map_err(|_| anyhow::anyhow!("app not found"))?;

    Ok(resolved_app.clone_app_for_operation())
}

fn apply_granted_permissions_to_app(
    app: &SharedSageApp,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<AppUpdateResult> {
    let (previous, new) = app.try_with_mut(|sage_app| {
        let previous = sage_app.common().granted_permissions().clone();

        sage_app
            .common_mut()
            .update_permissions(granted_permissions)
            .context("failed to update app permissions")?;

        let new = sage_app.common().granted_permissions().clone();

        Ok::<_, anyhow::Error>((previous, new))
    })?;

    write_installed_app_metadata(app).context("failed to persist app metadata")?;

    Ok(AppUpdateResult::new(GrantedPermissionsChange::diff(
        &previous,
        &new,
    )))
}

async fn emit_granted_permissions_change(
    app_handle: &AppHandle,
    app_id: &str,
    change: &GrantedPermissionsChange,
) {
    let capability_change = change.capabilities();

    if !capability_change.is_empty() {
        let _ = emit_runtime_event_to_app_id(
            app_handle,
            app_id,
            GrantedCapabilitiesChangeEvent::from_change(capability_change)
        )
            .await;
    }

    let network_change = change.network_whitelist();

    if !network_change.is_empty() {
        let _ = emit_runtime_event_to_app_id(
            app_handle,
            app_id,
            GrantedNetworkWhitelistChangeEvent::from_change(network_change)
        )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::capabilities::list::UserBridgeCapability;
    use crate::lifecycle::registry::{app_dir, read_installed_app_by_id};
    use crate::types::{
        InstalledSageAppStorage, SageApp, SageAppCommon, SageAppIdentity, SageAppManifestFile,
        SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot,
        SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions,
        UserSageApp, UserSageAppSource,
    };
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

    fn sample_app(base: &Path, app_id: &str) -> SharedSageApp {
        let app_dir = app_dir(base, app_id);
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::write(app_dir.join("index.html"), "test").unwrap();

        let requested_permissions = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [network_whitelist_entry("https", "required.example.com")],
                [network_whitelist_entry("wss", "optional.example.com")],
            ),
            SageRequestedCapabilities::new(
                [],
                [
                    UserBridgeCapability::WalletSendXch,
                    UserBridgeCapability::StoragePersistentWebview,
                ],
            ),
        )
            .unwrap();

        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: requested_permissions.clone(),
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 4).unwrap()],
            entry: Some("index.html".to_string()),
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

        SharedSageApp::new(SageApp::User(UserSageApp::new_installed(
            common,
            UserSageAppSource::url("https://example.com/app/").unwrap(),
        )))
    }

    #[test]
    fn update_app_permissions_persists_required_network_entries() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");

        write_installed_app_metadata(&app).unwrap();

        let granted = app
            .try_with(|app| SageGrantedPermissions::new(app.common().requested_permissions(), [], []))
            .unwrap();

        let update_result = apply_granted_permissions_to_app(&app, &granted).unwrap();

        assert_eq!(
            entries(update_result.change().network_whitelist().full.clone()),
            [network_whitelist_entry("https", "required.example.com")]
        );

        let reloaded = read_installed_app_by_id(dir.path(), "app-1").unwrap();

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

        let granted = app
            .try_with(|app| {
                app.common()
                    .granted_permissions()
                    .with_capability_added(
                        app.common().requested_permissions(),
                        UserBridgeCapability::WalletSendXch,
                    )
            })
            .unwrap();

        let update_result = apply_granted_permissions_to_app(&app, &granted).unwrap();

        let outcome =
            GrantCapabilityOutcome::from_update(UserBridgeCapability::WalletSendXch, &update_result);

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

        let reloaded = read_installed_app_by_id(dir.path(), "app-1").unwrap();

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
        let app = sample_app(dir.path(), "app-1");

        let granted = app
            .try_with(|app| {
                SageGrantedPermissions::new(
                    app.common().requested_permissions(),
                    [UserBridgeCapability::WalletSendXch],
                    [network_whitelist_entry("https", "required.example.com")],
                )
            })
            .unwrap();

        apply_granted_permissions_to_app(&app, &granted).unwrap();

        let same_granted = app
            .try_with(|app| {
                app.common()
                    .granted_permissions()
                    .with_capability_added(
                        app.common().requested_permissions(),
                        UserBridgeCapability::WalletSendXch,
                    )
            })
            .unwrap();

        let update_result = apply_granted_permissions_to_app(&app, &same_granted).unwrap();

        let outcome =
            GrantCapabilityOutcome::from_update(UserBridgeCapability::WalletSendXch, &update_result);

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

        let granted = app
            .try_with(|app| {
                app.common()
                    .granted_permissions()
                    .with_network_whitelist_entry_added(
                        app.common().requested_permissions(),
                        entry.clone(),
                    )
            })
            .unwrap();

        let update_result = apply_granted_permissions_to_app(&app, &granted).unwrap();

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

        let reloaded = read_installed_app_by_id(dir.path(), "app-1").unwrap();

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
        let app = sample_app(dir.path(), "app-1");

        let granted = app
            .try_with(|app| {
                SageGrantedPermissions::new(
                    app.common().requested_permissions(),
                    [],
                    [network_whitelist_entry("https", "required.example.com")],
                )
            })
            .unwrap();

        apply_granted_permissions_to_app(&app, &granted).unwrap();

        let entry = network_whitelist_entry("https", "required.example.com");

        let same_granted = app
            .try_with(|app| {
                app.common()
                    .granted_permissions()
                    .with_network_whitelist_entry_added(
                        app.common().requested_permissions(),
                        entry.clone(),
                    )
            })
            .unwrap();

        let update_result = apply_granted_permissions_to_app(&app, &same_granted).unwrap();

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
