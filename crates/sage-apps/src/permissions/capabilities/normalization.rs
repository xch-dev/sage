use std::collections::BTreeSet;
use anyhow::anyhow;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::types::SageRequestedCapabilities;

pub fn normalize_requested_capabilities(
    capabilities: &SageRequestedCapabilities,
) -> anyhow::Result<SageRequestedCapabilities> {
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



pub fn normalize_granted_capabilities(
    granted: &[UserBridgeCapability],
) -> anyhow::Result<Vec<UserBridgeCapability>> {
    let mut out = BTreeSet::new();

    for granted_capability in granted {
        let granted_capability_definition = get_user_capability_definition(*granted_capability);

        if granted_capability_definition.flags.user_grantable {
            out.insert(*granted_capability);
        }
    }

    Ok(out.into_iter().collect())
}
