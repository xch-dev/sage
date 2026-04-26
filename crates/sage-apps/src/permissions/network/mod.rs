use std::collections::BTreeSet;
use anyhow::{anyhow, Result};
use crate::permissions::network::normalization::normalize_network_entry;
use crate::types::{SageNetworkPermissionTarget, SageRequestedNetworkPermissions};

pub mod validation;
pub mod normalization;

pub fn normalize_and_validate_granted_network(
    requested: &SageRequestedNetworkPermissions,
    granted: &[SageNetworkPermissionTarget],
) -> Result<Vec<SageNetworkPermissionTarget>> {
    let requested_required = requested
        .whitelist
        .required
        .iter()
        .map(|entry| normalize_network_entry(&entry))
        .collect::<Result<BTreeSet<_>>>()?;

    let mut requested_optional = BTreeSet::new();

    for entry in &requested.whitelist.optional {
        let key = normalize_network_entry(&entry)?;

        if !requested_required.contains(&key) {
            requested_optional.insert(key);
        }
    }

    let mut result = requested_required;

    for entry in granted {
        let key = normalize_network_entry(&entry)?;

        if !result.contains(&key) && !requested_optional.contains(&key) {
            return Err(anyhow!(
                "granted network whitelist entry not requested in manifest: {}://{}",
                key.scheme,
                key.host
            ));
        }

        result.insert(key);
    }

    Ok(result.into_iter().collect())
}
