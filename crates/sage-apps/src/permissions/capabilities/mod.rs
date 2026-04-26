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
pub(crate) use types::CapabilityFlags;

use anyhow::Result;
use std::collections::BTreeSet;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
use crate::permissions::capabilities::validation::{validate_effective_granted_capabilities, validate_user_granted_capabilities};
use crate::types::{SageRequestedCapabilities};

pub(crate) fn normalize_and_validate_user_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    granted: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    validate_user_granted_capabilities(requested_capabilities, granted)?;

    let normalized = normalize_granted_capabilities(granted)?;

    validate_user_granted_capabilities(requested_capabilities, &normalized)?;

    Ok(normalized)
}

pub(crate) fn resolve_and_validate_effective_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    user_granted_capabilities: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    validate_user_granted_capabilities(requested_capabilities, user_granted_capabilities)?;

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

    let effective_vec: Vec<UserBridgeCapability> = effective.into_iter().collect();

    validate_effective_granted_capabilities(requested_capabilities, &effective_vec)?;

    Ok(effective_vec)
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

#[cfg(test)]
mod tests {
    use crate::permissions::resolve_and_validate_effective_granted_capabilities;
    use crate::permissions::tests::{auto_granted_capability, empty_requested_permissions};

    #[test]
    fn moving_non_user_grantable_capability_from_optional_to_required_still_auto_grants() {
        let auto = auto_granted_capability();

        let mut optional_requested = empty_requested_permissions();
        optional_requested.capabilities.optional = vec![auto];

        let optional_effective = resolve_and_validate_effective_granted_capabilities(
            &optional_requested.capabilities,
            &[],
        )
            .expect("optional auto grant should resolve");

        assert_eq!(optional_effective, vec![auto]);

        let mut required_requested = empty_requested_permissions();
        required_requested.capabilities.required = vec![auto];

        let required_effective = resolve_and_validate_effective_granted_capabilities(
            &required_requested.capabilities,
            &[],
        )
            .expect("required auto grant should resolve");

        assert_eq!(required_effective, vec![auto]);
    }

    #[test]
    fn removed_non_user_grantable_capability_is_no_longer_effective() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        let effective = resolve_and_validate_effective_granted_capabilities(&requested.capabilities, &[])
            .expect("expected auto grant before removal");

        assert_eq!(effective, vec![auto]);

        let removed_requested = empty_requested_permissions();

        let effective_after_removal =
            resolve_and_validate_effective_granted_capabilities(&removed_requested.capabilities, &[])
                .expect("expected permissions to resolve after removal");

        assert!(effective_after_removal.is_empty());
    }
}
