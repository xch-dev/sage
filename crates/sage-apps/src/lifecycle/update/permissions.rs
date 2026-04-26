use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::lifecycle::{parse_network_permission_target, read_installed_app_by_id, write_installed_app_metadata};
use crate::lifecycle::flags::{clear_storage_may_contain_secrets, get_app_flags};
use crate::lifecycle::update::types::{GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedCapabilitiesChange, GrantedNetworkWhitelistChange};
use crate::permissions::capabilities::definitions::{get_user_capability_definition};
use crate::permissions::capabilities::get_effective_granted_capabilities;
use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::permissions::network::normalize_and_validate_granted_network;
use crate::types::{SageGrantedNetworkPermissions, SageGrantedPermissions, SageNetworkPermissionTarget, UserSageApp};

pub fn update_app_permissions(
    base_path: &Path,
    app_id: &str,
    granted_permissions: SageGrantedPermissions,
    clear_storage_taint: bool,
) -> anyhow::Result<UserSageApp> {
    let mut app = read_installed_app_by_id(base_path, app_id)?;

    let normalized_capabilities = normalize_granted_capabilities(
        &granted_permissions.capabilities,
    )?;

    validate_granted_capabilities(
        &app.common.requested_permissions,
        &normalized_capabilities,
    )?;

    let effective_capabilities = get_effective_granted_capabilities(
        &app.common.requested_permissions,
        &normalized_capabilities,
    )?;

    let granted_network_whitelist = normalize_and_validate_granted_network(
        &app.common.active_snapshot.manifest.permissions.network,
        &granted_permissions.network.whitelist,
    )?;

    let mut permission_flags = get_app_flags(
        &effective_capabilities,
        Some(&app.common.capability_flags),
    )?;

    if clear_storage_taint {
        permission_flags = clear_storage_may_contain_secrets(&permission_flags);
    }

    app.common.granted_permissions = SageGrantedPermissions {
        capabilities: normalized_capabilities,
        network: SageGrantedNetworkPermissions {
            whitelist: granted_network_whitelist,
        },
    };

    let effective_capabilities = get_effective_granted_capabilities(
        &app.common.requested_permissions,
        &app.common.granted_permissions.capabilities,
    )?;

    app.common.capability_flags = get_app_flags(
        &effective_capabilities,
        Some(&permission_flags),
    )?;

    let app_dir = PathBuf::from(&app.common.app_dir);
    write_installed_app_metadata(&app, &app_dir)?;

    Ok(app)
}

fn sort_unique_network(
    values: impl IntoIterator<Item = SageNetworkPermissionTarget>,
) -> Vec<SageNetworkPermissionTarget> {
    let set: BTreeSet<SageNetworkPermissionTarget> = values.into_iter().collect();
    set.into_iter().collect()
}

fn requested_capability_set(app: &UserSageApp) -> BTreeSet<UserBridgeCapability> {
    let mut out = BTreeSet::new();
    out.extend(app.common.requested_permissions.capabilities.required.iter().copied());
    out.extend(app.common.requested_permissions.capabilities.optional.iter().copied());
    out
}

fn requested_network_set(app: &UserSageApp) -> anyhow::Result<BTreeSet<(String, String)>> {
    let mut out = BTreeSet::new();

    for entry in &app.common.requested_permissions.network.whitelist.required {
        let normalized = parse_network_permission_target(&format!(
            "{}://{}",
            entry.scheme, entry.host
        ))
            .map_err(anyhow::Error::msg)?;
        out.insert((normalized.scheme, normalized.host));
    }

    for entry in &app.common.requested_permissions.network.whitelist.optional {
        let normalized = parse_network_permission_target(&format!(
            "{}://{}",
            entry.scheme, entry.host
        ))
            .map_err(anyhow::Error::msg)?;
        out.insert((normalized.scheme, normalized.host));
    }

    Ok(out)
}

fn diff_capabilities(
    previous: &[UserBridgeCapability],
    next: &[UserBridgeCapability],
) -> GrantedCapabilitiesChange {
    let previous_set: BTreeSet<UserBridgeCapability> =
        previous.iter().copied().collect();
    let next_set: BTreeSet<UserBridgeCapability> =
        next.iter().copied().collect();

    GrantedCapabilitiesChange {
        removed: previous_set.difference(&next_set).copied().collect(),
        added: next_set.difference(&previous_set).copied().collect(),
        full: next.to_vec(),
    }
}

fn diff_network_whitelist(
    previous: &[SageNetworkPermissionTarget],
    next: &[SageNetworkPermissionTarget],
) -> GrantedNetworkWhitelistChange {
    let previous_set: BTreeSet<SageNetworkPermissionTarget> = previous.iter().cloned().collect();
    let next_set: BTreeSet<SageNetworkPermissionTarget> = next.iter().cloned().collect();

    GrantedNetworkWhitelistChange {
        removed: previous_set.difference(&next_set).cloned().collect(),
        added: next_set.difference(&previous_set).cloned().collect(),
        full: next.to_vec(),
    }
}

fn sort_unique_capabilities(
    capabilities: Vec<UserBridgeCapability>,
) -> Vec<UserBridgeCapability> {
    capabilities.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
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

    if !definition.flags.user_grantable {
        anyhow::bail!(
            "Capability is not user-grantable and cannot be persisted as a user grant: {}",
            capability.key()
        );
    }

    if app.common.granted_permissions.capabilities.contains(&capability) {
        let full_granted_capabilities =
            sort_unique_capabilities(app.common.granted_permissions.capabilities.clone());

        return Ok(GrantCapabilityOutcome::AlreadyGranted {
            capability,
            full_granted_capabilities,
        });
    }

    let mut next_capabilities = app.common.granted_permissions.capabilities.clone();
    next_capabilities.push(capability);
    next_capabilities = sort_unique_capabilities(next_capabilities);

    let updated = update_app_permissions(
        base_path,
        app_id,
        SageGrantedPermissions {
            capabilities: next_capabilities,
            network: app.common.granted_permissions.network.clone(),
        },
        false,
    )?;

    let change = diff_capabilities(
        &app.common.granted_permissions.capabilities,
        &updated.common.granted_permissions.capabilities,
    );

    Ok(GrantCapabilityOutcome::Granted { capability, change })
}

pub fn grant_requested_network_whitelist_entry_internal(
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkPermissionTarget,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    let normalized = parse_network_permission_target(&format!(
        "{}://{}",
        entry.scheme, entry.host
    ))
        .map_err(anyhow::Error::msg)?;
    let entry = SageNetworkPermissionTarget {
        scheme: normalized.scheme,
        host: normalized.host,
    };

    let requested = requested_network_set(&app)?;
    let entry_key = (entry.scheme.clone(), entry.host.clone());

    if !requested.contains(&entry_key) {
        anyhow::bail!(
            "Network whitelist entry was not requested by app manifest: {}://{}",
            entry.scheme,
            entry.host
        );
    }

    if app
        .common
        .granted_permissions
        .network
        .whitelist
        .iter()
        .any(|existing| existing == &entry)
    {
        let full_granted_network_whitelist =
            sort_unique_network(app.common.granted_permissions.network.whitelist.clone());

        return Ok(GrantNetworkWhitelistOutcome::AlreadyGranted {
            entry,
            full_granted_network_whitelist,
        });
    }

    let mut next_whitelist = app.common.granted_permissions.network.whitelist.clone();
    next_whitelist.push(entry.clone());
    next_whitelist = sort_unique_network(next_whitelist);

    let updated = update_app_permissions(
        base_path,
        app_id,
        SageGrantedPermissions {
            capabilities: app.common.granted_permissions.capabilities.clone(),
            network: SageGrantedNetworkPermissions {
                whitelist: next_whitelist,
            },
        },
        false,
    )?;

    let change = diff_network_whitelist(
        &app.common.granted_permissions.network.whitelist,
        &updated.common.granted_permissions.network.whitelist,
    );

    Ok(GrantNetworkWhitelistOutcome::Granted { entry, change })
}

pub fn update_app_permissions_with_change_internal(
    base_path: &Path,
    app_id: &str,
    granted_permissions: SageGrantedPermissions,
    clear_storage_taint: bool,
) -> anyhow::Result<(
    UserSageApp,
    GrantedCapabilitiesChange,
    GrantedNetworkWhitelistChange,
)> {
    let previous = read_installed_app_by_id(base_path, app_id)?;

    let updated = update_app_permissions(
        base_path,
        app_id,
        granted_permissions,
        clear_storage_taint,
    )?;

    let capability_change = diff_capabilities(
        &previous.common.granted_permissions.capabilities,
        &updated.common.granted_permissions.capabilities,
    );

    let network_change = diff_network_whitelist(
        &previous.common.granted_permissions.network.whitelist,
        &updated.common.granted_permissions.network.whitelist,
    );

    Ok((updated, capability_change, network_change))
}
