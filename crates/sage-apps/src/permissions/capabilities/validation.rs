use std::collections::BTreeSet;
use anyhow::{anyhow, Result};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::types::{SageRequestedCapabilities};

pub(in crate::permissions) fn validate_granted_capabilities(
    requested_capabilities: &SageRequestedCapabilities,
    granted: &[UserBridgeCapability],
) -> Result<()> {
    let mut allowed_capabilities = BTreeSet::new();
    allowed_capabilities.extend(requested_capabilities.required.iter().copied());
    allowed_capabilities.extend(requested_capabilities.optional.iter().copied());

    let granted_set: BTreeSet<_> = granted.iter().copied().collect();

    for granted_capability in &granted_set {
        if !allowed_capabilities.contains(granted_capability) {
            return Err(anyhow!(
                "granted capabilities not requested in manifest: {}",
                granted_capability.key()
            ));
        }
    }

    for required_capability in &requested_capabilities.required {
        let required_capability_definition = get_user_capability_definition(*required_capability);

        if required_capability_definition.flags.user_grantable && !granted_set.contains(required_capability) {
            return Err(anyhow!("missing required capability: {}", required_capability.key()));
        }
    }

    Ok(())
}
