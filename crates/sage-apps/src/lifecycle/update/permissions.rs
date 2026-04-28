use std::collections::BTreeSet;
use std::path::Path;

use crate::bridge::capabilities::UserBridgeCapability;
use crate::capabilities::get_user_capability_definition;
use crate::lifecycle::update::types::{
    GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedCapabilitiesChange,
    GrantedNetworkWhitelistChange,
};
use crate::lifecycle::{read_installed_app_by_id, write_installed_app_metadata};
use crate::types::{SageGrantedPermissions, SageNetworkWhitelistEntry, UserSageApp};

pub fn update_app_permissions(
    base_path: &Path,
    app_id: &str,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<UserSageApp> {
    let mut app = read_installed_app_by_id(base_path, app_id)?;

    app.common_mut().update_permissions(granted_permissions)?;

    write_installed_app_metadata(&app, &app.app_path())?;

    Ok(app)
}

fn sort_unique_network(
    values: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
) -> Vec<SageNetworkWhitelistEntry> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sort_unique_capabilities(
    values: impl IntoIterator<Item = UserBridgeCapability>,
) -> Vec<UserBridgeCapability> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn requested_capability_set(app: &UserSageApp) -> BTreeSet<UserBridgeCapability> {
    app.common()
        .requested_permissions()
        .capabilities()
        .required()
        .chain(
            app.common()
                .requested_permissions()
                .capabilities()
                .optional(),
        )
        .copied()
        .collect()
}

fn granted_capabilities(app: &UserSageApp) -> Vec<UserBridgeCapability> {
    app.common()
        .granted_permissions()
        .capabilities()
        .copied()
        .collect()
}

fn granted_network_whitelist(app: &UserSageApp) -> Vec<SageNetworkWhitelistEntry> {
    app.common()
        .granted_permissions()
        .network()
        .whitelist()
        .cloned()
        .collect()
}

fn diff_capabilities(
    previous: &[UserBridgeCapability],
    next: &[UserBridgeCapability],
) -> GrantedCapabilitiesChange {
    let previous_set: BTreeSet<UserBridgeCapability> = previous.iter().copied().collect();
    let next_set: BTreeSet<UserBridgeCapability> = next.iter().copied().collect();

    GrantedCapabilitiesChange {
        removed: previous_set.difference(&next_set).copied().collect(),
        added: next_set.difference(&previous_set).copied().collect(),
        full: next.to_vec(),
    }
}

fn diff_network_whitelist(
    previous: &[SageNetworkWhitelistEntry],
    next: &[SageNetworkWhitelistEntry],
) -> GrantedNetworkWhitelistChange {
    let previous_set: BTreeSet<SageNetworkWhitelistEntry> = previous.iter().cloned().collect();
    let next_set: BTreeSet<SageNetworkWhitelistEntry> = next.iter().cloned().collect();

    GrantedNetworkWhitelistChange {
        removed: previous_set.difference(&next_set).cloned().collect(),
        added: next_set.difference(&previous_set).cloned().collect(),
        full: next.to_vec(),
    }
}

pub fn grant_requested_capability_internal(
    base_path: &Path,
    app_id: &str,
    capability: UserBridgeCapability,
) -> anyhow::Result<GrantCapabilityOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let requested = requested_capability_set(&app);
    if !requested.contains(&capability) {
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

    let previous_capabilities = granted_capabilities(&app);

    if previous_capabilities.contains(&capability) {
        return Ok(GrantCapabilityOutcome::AlreadyGranted {
            capability,
            full_granted_capabilities: sort_unique_capabilities(previous_capabilities),
        });
    }

    let next_capabilities =
        sort_unique_capabilities(previous_capabilities.iter().copied().chain([capability]));

    let previous_network = granted_network_whitelist(&app);

    let granted_permissions = SageGrantedPermissions::new(
        app.common().requested_permissions(),
        next_capabilities,
        previous_network.clone(),
    )?;

    let updated = update_app_permissions(base_path, app_id, &granted_permissions)?;

    let updated_capabilities = granted_capabilities(&updated);
    let change = diff_capabilities(&previous_capabilities, &updated_capabilities);

    Ok(GrantCapabilityOutcome::Granted { capability, change })
}

pub fn grant_requested_network_whitelist_entry_internal(
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    if !app
        .common()
        .requested_permissions()
        .network()
        .whitelist()
        .is_allowed(entry)
    {
        anyhow::bail!(
            "Network whitelist entry was not requested by app manifest: {}",
            entry.as_permission_string(),
        );
    }

    let previous_whitelist = granted_network_whitelist(&app);

    if previous_whitelist.iter().any(|existing| existing == entry) {
        return Ok(GrantNetworkWhitelistOutcome::AlreadyGranted {
            entry: entry.clone(),
            full_granted_network_whitelist: sort_unique_network(previous_whitelist),
        });
    }

    let next_whitelist =
        sort_unique_network(previous_whitelist.iter().cloned().chain([entry.clone()]));

    let granted_permissions = SageGrantedPermissions::new(
        app.common().requested_permissions(),
        granted_capabilities(&app),
        next_whitelist,
    )?;

    let updated = update_app_permissions(base_path, app_id, &granted_permissions)?;

    let updated_whitelist = granted_network_whitelist(&updated);
    let change = diff_network_whitelist(&previous_whitelist, &updated_whitelist);

    Ok(GrantNetworkWhitelistOutcome::Granted {
        entry: entry.clone(),
        change,
    })
}

pub fn update_app_permissions_with_change_internal(
    base_path: &Path,
    app_id: &str,
    granted_permissions: &SageGrantedPermissions,
) -> anyhow::Result<(
    UserSageApp,
    GrantedCapabilitiesChange,
    GrantedNetworkWhitelistChange,
)> {
    let previous = read_installed_app_by_id(base_path, app_id)?;

    let previous_capabilities = granted_capabilities(&previous);
    let previous_network = granted_network_whitelist(&previous);

    let updated = update_app_permissions(base_path, app_id, granted_permissions)?;

    let updated_capabilities = granted_capabilities(&updated);
    let updated_network = granted_network_whitelist(&updated);

    let capability_change = diff_capabilities(&previous_capabilities, &updated_capabilities);
    let network_change = diff_network_whitelist(&previous_network, &updated_network);

    Ok((updated, capability_change, network_change))
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
            UserSageAppSource::Url {
                app_url: "https://example.com/app/".into(),
                manifest_url: "https://example.com/app/sage-manifest.json".into(),
            },
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

        let updated = update_app_permissions(dir.path(), app.common().id(), &granted).unwrap();

        assert_eq!(
            entries(
                updated
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
