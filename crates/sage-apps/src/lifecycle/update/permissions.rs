use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::bridge::capabilities::UserBridgeCapability;
use crate::lifecycle::{
    read_installed_app_by_id, write_installed_app_metadata,
};
use crate::lifecycle::update::types::{
    GrantCapabilityOutcome, GrantNetworkWhitelistOutcome, GrantedCapabilitiesChange,
    GrantedNetworkWhitelistChange,
};
use crate::permissions::get_user_capability_definition;
use crate::types::{
    SageGrantedPermissions, SageNetworkWhitelistEntry, UserSageApp,
};

pub fn update_app_permissions(
    base_path: &Path,
    app_id: &str,
    granted_permissions: SageGrantedPermissions,
) -> anyhow::Result<UserSageApp> {
    let mut app = read_installed_app_by_id(base_path, app_id)?;

    app.common.update_permissions(&granted_permissions)?;

    let app_dir = PathBuf::from(&app.common.app_dir);
    write_installed_app_metadata(&app, &app_dir)?;

    Ok(app)
}

fn sort_unique_network(
    values: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
) -> Vec<SageNetworkWhitelistEntry> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn sort_unique_capabilities(
    values: impl IntoIterator<Item = UserBridgeCapability>,
) -> Vec<UserBridgeCapability> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn requested_capability_set(app: &UserSageApp) -> BTreeSet<UserBridgeCapability> {
    app.common
        .requested_permissions
        .capabilities
        .required()
        .chain(app.common.requested_permissions.capabilities.optional())
        .copied()
        .collect()
}

fn granted_capabilities(app: &UserSageApp) -> Vec<UserBridgeCapability> {
    app.common.granted_permissions.capabilities().copied().collect()
}

fn granted_network_whitelist(app: &UserSageApp) -> Vec<SageNetworkWhitelistEntry> {
    app.common
        .granted_permissions
        .network()
        .whitelist()
        .cloned()
        .collect()
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
    previous: &[SageNetworkWhitelistEntry],
    next: &[SageNetworkWhitelistEntry],
) -> GrantedNetworkWhitelistChange {
    let previous_set: BTreeSet<SageNetworkWhitelistEntry> =
        previous.iter().cloned().collect();
    let next_set: BTreeSet<SageNetworkWhitelistEntry> =
        next.iter().cloned().collect();

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
    if !definition.flags.user_grantable {
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

    let next_capabilities = sort_unique_capabilities(
        previous_capabilities
            .iter()
            .copied()
            .chain([capability]),
    );

    let previous_network = granted_network_whitelist(&app);

    let granted_permissions = SageGrantedPermissions::new(
        &app.common.requested_permissions,
        next_capabilities.clone(),
        previous_network.clone(),
    )?;

    let updated = update_app_permissions(base_path, app_id, granted_permissions)?;

    let updated_capabilities = granted_capabilities(&updated);

    let change = diff_capabilities(
        &previous_capabilities,
        &updated_capabilities,
    );

    Ok(GrantCapabilityOutcome::Granted { capability, change })
}

pub fn grant_requested_network_whitelist_entry_internal(
    base_path: &Path,
    app_id: &str,
    entry: &SageNetworkWhitelistEntry,
) -> anyhow::Result<GrantNetworkWhitelistOutcome> {
    let app = read_installed_app_by_id(base_path, app_id)?;

    if !app
        .common
        .requested_permissions
        .network
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

    let next_whitelist = sort_unique_network(
        previous_whitelist
            .iter()
            .cloned()
            .chain([entry.clone()]),
    );

    let granted_permissions = SageGrantedPermissions::new(
        &app.common.requested_permissions,
        granted_capabilities(&app),
        next_whitelist.clone(),
    )?;

    let updated = update_app_permissions(base_path, app_id, granted_permissions)?;

    let updated_whitelist = granted_network_whitelist(&updated);

    let change = diff_network_whitelist(
        &previous_whitelist,
        &updated_whitelist,
    );

    Ok(GrantNetworkWhitelistOutcome::Granted {
        entry: entry.clone(),
        change,
    })
}

pub fn update_app_permissions_with_change_internal(
    base_path: &Path,
    app_id: &str,
    granted_permissions: SageGrantedPermissions,
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

    let capability_change =
        diff_capabilities(&previous_capabilities, &updated_capabilities);

    let network_change =
        diff_network_whitelist(&previous_network, &updated_network);

    Ok((updated, capability_change, network_change))
}
