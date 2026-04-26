use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use crate::types::{SageNetworkPermissionTarget, SageRequestedNetworkPermissions};

pub(in crate::permissions) fn validate_granted_network(
    requested: &SageRequestedNetworkPermissions,
    granted: &[SageNetworkPermissionTarget],
) -> Result<()> {
    let allowed: BTreeSet<_> = requested
        .whitelist
        .required
        .iter()
        .chain(requested.whitelist.optional.iter())
        .cloned()
        .collect();

    for entry in granted {
        if !allowed.contains(entry) {
            return Err(anyhow!(
                "granted network whitelist entry not requested in manifest: {}://{}",
                entry.scheme,
                entry.host
            ));
        }
    }

    Ok(())
}
