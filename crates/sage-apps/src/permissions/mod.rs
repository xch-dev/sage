pub mod tests;
pub mod validation;
pub mod capabilities;
pub mod network;
pub mod normalization;

pub use validation::*;

use anyhow::Result;
use crate::permissions::capabilities::normalize_and_validate_granted_capabilities;
use crate::permissions::network::normalize_and_validate_granted_network;
use crate::permissions::normalization::normalize_requested_permissions;
use crate::types::{SageGrantedNetworkPermissions, SageGrantedPermissions, SageRequestedPermissions};

pub fn normalize_and_validate_requested_permissions(
    permissions: &SageRequestedPermissions,
) -> Result<SageRequestedPermissions> {
    let normalized_requested = normalize_requested_permissions(permissions)?;

    validate_requested_permission(&normalized_requested)?;

    Ok(normalized_requested)
}

pub(super) fn normalize_and_validate_granted_permissions(
    requested: &SageRequestedPermissions,
    granted: SageGrantedPermissions,
) -> Result<SageGrantedPermissions> {
    Ok(SageGrantedPermissions {
        capabilities: normalize_and_validate_granted_capabilities(
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
