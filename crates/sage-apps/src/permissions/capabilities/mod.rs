use anyhow::Result;
use std::collections::BTreeSet;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::types::{SageRequestedCapabilities};

pub mod definitions;
pub mod normalization;
pub(in crate::permissions) mod validation;
pub mod types;

pub(crate) fn normalize_and_validate_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    granted: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    let normalized = normalize_granted_capabilities(&granted)?;

    validate_granted_capabilities(requested_capabilities, &normalized)?;

    Ok(normalized)
}

pub(crate) fn get_and_validate_effective_granted_capabilities(
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
