use anyhow::{anyhow, Result};
use crate::permissions::capabilities::types::CapabilityFlags;
use crate::types::{SageRequestedPermissions};

pub fn validate_requested_permission(
    permissions: &SageRequestedPermissions,
) -> Result<()> {
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
