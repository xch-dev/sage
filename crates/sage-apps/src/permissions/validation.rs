use anyhow::{anyhow, Result as AnyResult};
use crate::permissions::capabilities::normalization::{normalize_granted_capabilities, normalize_requested_capabilities};
use crate::permissions::capabilities::types::CapabilityFlags;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::permissions::network::normalization::normalize_requested_network;
use crate::permissions::network::normalize_and_validate_granted_network;
use crate::types::{SageGrantedPermissions, SageRequestedPermissions};

pub fn validate_requested_permission(
    permissions: &SageRequestedPermissions,
) -> AnyResult<()> {
    let mut requested = Vec::new();
    requested.extend(permissions.capabilities.required.iter().copied());
    requested.extend(permissions.capabilities.optional.iter().copied());


    let requested_capability_flags = CapabilityFlags::from_capabilities(&requested)?;

    if requested_capability_flags.externally_observable && requested_capability_flags.accesses_sensitive_secret {
        return Err(anyhow!(
            "requested permissions cannot include both externally observable and sensitive secret access permissions"
        ));
    }

    Ok(())
}

pub fn normalize_and_validate_requested_permissions(
    permissions: &SageRequestedPermissions,
) -> AnyResult<SageRequestedPermissions> {
    let normalized = SageRequestedPermissions {
        network: normalize_requested_network(&permissions.network)?,
        capabilities: normalize_requested_capabilities(&permissions.capabilities)?,
    };

    validate_requested_permission(&normalized)?;
    Ok(normalized)
}

pub fn normalize_and_validate_granted_permissions(
    requested: &SageRequestedPermissions,
    granted: SageGrantedPermissions,
) -> AnyResult<SageGrantedPermissions> {
    let normalized_capabilities = normalize_granted_capabilities(&granted.capabilities)?;

    validate_granted_capabilities(requested, &normalized_capabilities)?;

    let whitelist = normalize_and_validate_granted_network(
        &requested.network,
        &granted.network.whitelist,
    )?;

    Ok(SageGrantedPermissions {
        capabilities: normalized_capabilities,
        network: crate::types::SageGrantedNetworkPermissions { whitelist },
    })
}
