use std::collections::BTreeSet;
use anyhow::anyhow;
use crate::types::{SageNetworkPermissionTarget, SageRequestedNetworkPermissions};

pub fn normalize_requested_network(
    network_permissions: &SageRequestedNetworkPermissions,
) -> anyhow::Result<SageRequestedNetworkPermissions> {
    let required = normalize_requested_network_entries(&network_permissions.whitelist.required)?;
    let required_keys = required.iter().cloned().collect::<BTreeSet<_>>();

    let optional = normalize_requested_network_entries(&network_permissions.whitelist.optional)?
        .into_iter()
        .filter(|entry| !required_keys.contains(entry))
        .collect();

    Ok(SageRequestedNetworkPermissions {
        whitelist: crate::types::SageRequestedNetworkWhitelist { required, optional },
    })
}

pub fn normalize_network_entry(
    entry: &SageNetworkPermissionTarget,
) -> anyhow::Result<SageNetworkPermissionTarget> {
    let scheme = entry.scheme.trim().to_ascii_lowercase();
    let host = entry.host.trim().to_ascii_lowercase();

    if scheme.is_empty() {
        return Err(anyhow!("network whitelist entry is missing scheme"));
    }

    if host.is_empty() {
        return Err(anyhow!("network whitelist entry is missing host"));
    }

    Ok(SageNetworkPermissionTarget { scheme, host })
}

fn normalize_requested_network_entries(
    entries: &[SageNetworkPermissionTarget],
) -> anyhow::Result<Vec<SageNetworkPermissionTarget>> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::new();

    for entry in entries {
        let normalized_entry = normalize_network_entry(&entry)?;
        if seen.insert(normalized_entry.clone()) {
            normalized.push(normalized_entry);
        }
    }

    normalized.sort_by(|a, b| {
        let a_key = format!("{}://{}", a.scheme, a.host);
        let b_key = format!("{}://{}", b.scheme, b.host);
        a_key.cmp(&b_key)
    });

    Ok(normalized)
}
