mod definitions;
mod normalization;
mod validation;
mod types;

pub(crate) use definitions::{
    get_user_capability_definition, get_system_capability_definition,
    user_capability_definition_view,
    user_registry
};
pub(super) use normalization::normalize_requested_capabilities;
pub(super) use validation::validate_granted_capabilities;
pub(crate) use types::CapabilityFlags;

use anyhow::Result;
use std::collections::BTreeSet;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
use crate::types::{SageRequestedCapabilities};

pub(crate) fn normalize_and_validate_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    granted: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    let normalized = normalize_granted_capabilities(&granted)?;

    validate_granted_capabilities(requested_capabilities, &normalized)?;

    Ok(normalized)
}

pub(crate) fn resolve_effective_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    user_granted_capabilities: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    validate_granted_capabilities(requested_capabilities, user_granted_capabilities)?;

    let mut effective = BTreeSet::new();
    effective.extend(user_granted_capabilities.iter().copied());

    let requested_capabilities_chain = requested_capabilities
        .required
        .iter()
        .chain(requested_capabilities.optional.iter());

    for requested_capability in requested_capabilities_chain {
        let definition = get_user_capability_definition(*requested_capability);

        if !definition.flags.user_grantable {
            effective.insert(*requested_capability);
        }
    }

    Ok(effective.into_iter().collect())
}

pub(crate) fn requested_user_grantable_capabilities(
    requested: &SageRequestedCapabilities,
) -> Vec<UserBridgeCapability> {
    let mut caps: Vec<_> = requested
        .required
        .iter()
        .chain(requested.optional.iter())
        .copied()
        .filter(|cap| {
            get_user_capability_definition(*cap)
                .flags
                .user_grantable
        })
        .collect();

    caps.sort();
    caps.dedup();
    caps
}
