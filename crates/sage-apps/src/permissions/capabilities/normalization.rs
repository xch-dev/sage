use std::collections::BTreeSet;
use anyhow::{anyhow, Result};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::types::SageRequestedCapabilities;

pub(in crate::permissions) fn normalize_requested_capabilities(
    capabilities: &SageRequestedCapabilities,
) -> Result<SageRequestedCapabilities> {
    let mut required = BTreeSet::new();
    let mut optional = BTreeSet::new();

    for capability in &capabilities.required {
        let definition = get_user_capability_definition(*capability);

        if !definition.flags.requestable_by_app {
            return Err(anyhow!(
                "capability is not requestable by apps: {}",
                capability.key()
            ));
        }

        required.insert(*capability);
    }

    for capability in &capabilities.optional {
        let definition = get_user_capability_definition(*capability);

        if !definition.flags.requestable_by_app {
            return Err(anyhow!(
                "capability is not requestable by apps: {}",
                capability.key()
            ));
        }

        if !required.contains(capability) {
            optional.insert(*capability);
        }
    }

    Ok(SageRequestedCapabilities {
        required: required.into_iter().collect(),
        optional: optional.into_iter().collect(),
    })
}

pub(in crate::permissions::capabilities) fn normalize_granted_capabilities(
    granted: &[UserBridgeCapability],
) -> Result<Vec<UserBridgeCapability>> {
    let mut out = BTreeSet::new();

    for granted_capability in granted {
        let granted_capability_definition = get_user_capability_definition(*granted_capability);

        if granted_capability_definition.flags.user_grantable {
            out.insert(*granted_capability);
        }
    }

    Ok(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use crate::permissions::capabilities::resolve_effective_granted_capabilities;
    use crate::permissions::capabilities::normalization::normalize_granted_capabilities;

    #[test]
    fn normalize_user_granted_capabilities_strips_non_user_grantable_capability() {
        let auto = crate::permissions::tests::tests::auto_granted_capability();

        let normalized = normalize_granted_capabilities(&[auto])
            .expect("normalization should tolerate and strip stale non-user-grantable grants");

        assert!(normalized.is_empty());

        let mut requested = crate::permissions::tests::tests::empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        let effective = resolve_effective_granted_capabilities(&requested.capabilities, &normalized)
            .expect("auto capability should still be effective");

        assert_eq!(effective, vec![auto]);
    }
}
