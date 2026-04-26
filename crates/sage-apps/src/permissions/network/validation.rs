use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use crate::types::{SageNetworkPermissionTarget, SageRequestedNetworkPermissions};

pub(in crate::permissions) fn validate_granted_network(
    requested: &SageRequestedNetworkPermissions,
    granted: &[SageNetworkPermissionTarget],
) -> Result<()> {
    let requested_required = requested
        .whitelist
        .required
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let requested_optional = requested
        .whitelist
        .optional
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    for entry in granted {
        if !requested_required.contains(entry) && !requested_optional.contains(entry) {
            return Err(anyhow!(
                "granted network whitelist entry not requested in manifest: {}://{}",
                entry.scheme,
                entry.host
            ));
        }
    }

    Ok(())
}
