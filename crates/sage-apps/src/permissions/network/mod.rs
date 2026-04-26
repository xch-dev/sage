mod validation;
mod normalization;

pub(super) use validation::validate_granted_network;
pub(super) use normalization::normalize_requested_network;

use anyhow::Result;
use crate::permissions::network::normalization::{normalize_granted_network};
use crate::types::{SageNetworkPermissionTarget, SageRequestedNetworkPermissions};

pub(crate) fn normalize_and_validate_granted_network(
    requested: &SageRequestedNetworkPermissions,
    granted: &[SageNetworkPermissionTarget],
) -> Result<Vec<SageNetworkPermissionTarget>> {
    let requested = normalize_requested_network(requested)?;
    let granted = normalize_granted_network(granted)?;

    validate_granted_network(&requested, &granted)?;

    Ok(granted)
}
