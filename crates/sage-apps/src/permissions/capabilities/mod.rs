use anyhow::Result;
use std::collections::BTreeSet;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::types::SageRequestedPermissions;

pub mod definitions;
pub mod normalization;
pub mod validation;
pub mod types;

pub(crate) fn get_effective_granted_capabilities(
    requested_permissions: &SageRequestedPermissions,
    user_granted_capabilities: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    validate_granted_capabilities(requested_permissions, user_granted_capabilities)?;

    let mut effective = BTreeSet::new();
    effective.extend(user_granted_capabilities.iter().copied());

    for capability in requested_permissions
        .capabilities
        .required
        .iter()
        .chain(requested_permissions.capabilities.optional.iter())
    {
        let definition = get_user_capability_definition(*capability);

        if !definition.flags.user_grantable {
            effective.insert(*capability);
        }
    }

    Ok(effective.into_iter().collect())
}
