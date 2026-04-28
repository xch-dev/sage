use std::path::Path;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::capabilities::get_user_capability_definition;
use crate::lifecycle::update::types::{
    AppUpdateResult, GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedPermissionsChange,
};
use crate::lifecycle::{read_installed_app_by_id, write_installed_app_metadata};
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry};
use crate::utils::sorted_unique;

pub fn update_app_permissions(
    base_path: &Path,
    app_id: &str,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<AppUpdateResult> {
    let mut app = read_installed_app_by_id(base_path, app_id)?;

    let previous_permissions = app.common().granted_permissions().clone();

    app.common_mut().update_permissions(granted_permissions)?;

    write_installed_app_metadata(&app, &app.app_path())?;

    let change =
        GrantedPermissionsChange::diff(&previous_permissions, app.common().granted_permissions());

    Ok(AppUpdateResult::new(app, change))
}

pub fn grant_requested_capability_internal(
    base_path: &Path,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<GrantCapabilityOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let is_requested = app
        .common()
        .requested_permissions()
        .capabilities()
        .contains(capability);

    if !is_requested {
        anyhow::bail!(
            "Capability was not requested by app manifest: {}",
            capability.key()
        );
    }

    let definition = get_user_capability_definition(capability);
    if !definition.flags().user_grantable() {
        anyhow::bail!(
            "Capability is not user-grantable and cannot be persisted as a user grant: {}",
            capability.key()
        );
    }

    let previous_capabilities = app.common().granted_permissions().capabilities_vec();

    if previous_capabilities.contains(&capability) {
        return Ok(GrantCapabilityOutcome::AlreadyGranted {
            capability,
            full_granted_capabilities: sorted_unique(previous_capabilities),
        });
    }

    let next_capabilities =
        sorted_unique(previous_capabilities.iter().copied().chain([capability]));

    let previous_network = app.common().granted_permissions().network_whitelist_vec();

    let granted_permissions = SageGrantedPermissions::new(
        app.common().requested_permissions(),
        next_capabilities,
        previous_network.clone(),
    )?;

    let update_result = update_app_permissions(base_path, app_id, &granted_permissions)?;

    Ok(GrantCapabilityOutcome::Granted {
        capability,
        change: update_result.change().capabilities().clone(),
    })
}

pub fn grant_requested_network_whitelist_entry_internal(
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let is_whitelist_entry_requested = app
        .common()
        .requested_permissions()
        .network()
        .whitelist()
        .is_allowed(entry);
    if !is_whitelist_entry_requested {
        anyhow::bail!(
            "Network whitelist entry was not requested by app manifest: {}",
            entry.as_permission_string(),
        );
    }

    let previous_whitelist = app.common().granted_permissions().network_whitelist_vec();

    if previous_whitelist.iter().any(|existing| existing == entry) {
        return Ok(GrantNetworkWhitelistOutcome::AlreadyGranted {
            entry: entry.clone(),
            full_granted_network_whitelist: sorted_unique(previous_whitelist),
        });
    }

    let next_whitelist = sorted_unique(previous_whitelist.iter().cloned().chain([entry.clone()]));

    let granted_permissions = SageGrantedPermissions::new(
        app.common().requested_permissions(),
        app.common().granted_permissions().capabilities_vec(),
        next_whitelist,
    )?;

    let update_result = update_app_permissions(base_path, app_id, &granted_permissions)?;

    Ok(GrantNetworkWhitelistOutcome::Granted {
        entry: entry.clone(),
        change: update_result.change().network_whitelist().clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::registry::{
        app_dir, read_installed_app_by_id, write_installed_app_metadata,
    };
    use crate::types::{
        InstalledSageAppStorage, SageAppCommon, SageAppManifestFile, SageAppPackageManifest,
        SageAppPackageManifestParts, SageAppSnapshot, SageGrantedPermissions,
        SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions, UserSageApp, UserSageAppSource,
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
            app_id,
            app_id,
            app_dir.to_string_lossy().to_string(),
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
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let granted =
            SageGrantedPermissions::new(app.common().requested_permissions(), [], []).unwrap();

        let update_result =
            update_app_permissions(dir.path(), app.common().id(), &granted).unwrap();

        assert_eq!(
            entries(
                update_result
                    .app()
                    .common()
                    .granted_permissions()
                    .network()
                    .whitelist()
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
                    .whitelist()
                    .cloned()
            ),
            [network_whitelist_entry("https", "required.example.com")]
        );
    }

    #[test]
    fn update_app_permissions_rejects_unrequested_capability() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let err = SageGrantedPermissions::new(
            app.common().requested_permissions(),
            [UserBridgeCapability::WalletSendXchAutoSubmit],
            [],
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("not requested in manifest"));
        assert!(err.contains(UserBridgeCapability::WalletSendXchAutoSubmit.key()));
    }

    #[test]
    fn grant_requested_capability_grants_optional_capability() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let outcome = grant_requested_capability_internal(
            dir.path(),
            app.common().id(),
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

        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let outcome = grant_requested_capability_internal(
            dir.path(),
            app.common().id(),
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
                panic!("expected already-granted outcome");
            }
        }
    }

    #[test]
    fn grant_requested_capability_rejects_unrequested_capability() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let err = grant_requested_capability_internal(
            dir.path(),
            app.common().id(),
            UserBridgeCapability::WalletSendXchAutoSubmit,
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("Capability was not requested by app manifest")
        );
    }

    #[test]
    fn grant_requested_network_whitelist_entry_grants_optional_entry() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let outcome = grant_requested_network_whitelist_entry_internal(
            dir.path(),
            app.common().id(),
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

        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let outcome = grant_requested_network_whitelist_entry_internal(
            dir.path(),
            app.common().id(),
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
                panic!("expected already-granted outcome");
            }
        }
    }

    #[test]
    fn grant_requested_network_whitelist_entry_rejects_unrequested_entry() {
        let dir = tempdir().unwrap();
        let app = sample_app(dir.path(), "app-1");
        let app_path = app_dir(dir.path(), app.common().id());
        write_installed_app_metadata(&app, &app_path).unwrap();

        let err = grant_requested_network_whitelist_entry_internal(
            dir.path(),
            app.common().id(),
            &network_whitelist_entry("https", "evil.example.com"),
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("Network whitelist entry was not requested by app manifest")
        );
    }
}
