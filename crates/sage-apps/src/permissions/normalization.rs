use anyhow::Result;
use crate::permissions::capabilities::normalize_requested_capabilities;
use crate::permissions::network::normalization::normalize_requested_network;
use crate::types::SageRequestedPermissions;

pub fn normalize_requested_permissions(
    permissions: &SageRequestedPermissions,
) -> Result<SageRequestedPermissions> {
    Ok(SageRequestedPermissions {
        network: normalize_requested_network(&permissions.network)?,
        capabilities: normalize_requested_capabilities(&permissions.capabilities)?,
    })
}
