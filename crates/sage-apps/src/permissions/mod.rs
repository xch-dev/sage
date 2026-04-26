pub mod tests;
mod validation;
mod capabilities;
mod network;
mod normalization;

pub(crate) use capabilities::{
    get_user_capability_definition,
    normalize_and_validate_user_granted_capabilities,
    resolve_and_validate_effective_granted_capabilities,
    requested_user_grantable_capabilities,
    user_capability_definition_view,
    user_registry,
    get_system_capability_definition,
    CapabilityFlags,
};
pub(crate) use network::normalize_and_validate_granted_network;

use anyhow::Result;
use crate::permissions::normalization::normalize_requested_permissions;
use crate::permissions::validation::validate_requested_permission;
use crate::types::{SageGrantedNetworkPermissions, SageGrantedPermissions, SageRequestedPermissions};

pub(crate) fn normalize_and_validate_requested_permissions(
    permissions: &SageRequestedPermissions,
) -> Result<SageRequestedPermissions> {
    let normalized_requested = normalize_requested_permissions(permissions)?;

    validate_requested_permission(&normalized_requested)?;

    Ok(normalized_requested)
}

pub(crate) fn normalize_and_validate_granted_permissions(
    requested: &SageRequestedPermissions,
    granted: SageGrantedPermissions,
) -> Result<SageGrantedPermissions> {
    Ok(SageGrantedPermissions {
        capabilities: normalize_and_validate_user_granted_capabilities(
            &requested.capabilities,
            &granted.capabilities
        )?,
        network: SageGrantedNetworkPermissions {
            whitelist: normalize_and_validate_granted_network(
                &requested.network,
                &granted.network.whitelist,
            )?
        },
    })
}
