use std::collections::BTreeSet;

use crate::capabilities::list::UserBridgeCapability;
use crate::capabilities::{CapabilityFlags, get_user_capability_definition};
use crate::types::network::SageNetworkWhitelistEntry;
use crate::types::permissions::SageRequestedCapabilities;

pub fn validate_permissions_policy(
    capabilities: impl IntoIterator<Item = UserBridgeCapability>,
    network: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    context: &str,
) -> anyhow::Result<()> {
    let capability_flags = capabilities
        .into_iter()
        .fold(CapabilityFlags::EMPTY, |flags, cap| {
            flags.union(get_user_capability_definition(cap).flags())
        });

    let has_secret_access = capability_flags.accesses_sensitive_secret();
    let has_external_access =
        capability_flags.externally_observable() || network.into_iter().next().is_some();

    if has_secret_access && has_external_access {
        anyhow::bail!("{context} cannot include both external access and sensitive secret access");
    }

    Ok(())
}

pub fn validate_requested_capabilities_are_requestable(
    capabilities: &SageRequestedCapabilities,
) -> anyhow::Result<()> {
    for capability in capabilities.required().chain(capabilities.optional()) {
        let definition = get_user_capability_definition(*capability);

        if !definition.flags().requestable_by_app() {
            anyhow::bail!(
                "capability is not requestable by app manifest: {}",
                capability.key()
            );
        }
    }

    Ok(())
}

pub fn validate_network_id(network_id: &str) -> anyhow::Result<()> {
    if network_id.trim() != network_id {
        anyhow::bail!("network whitelist network id must not contain leading or trailing whitespace");
    }

    if network_id.is_empty() {
        anyhow::bail!("network whitelist network id cannot be empty");
    }

    if !matches!(network_id, "mainnet" | "testnet11") {
        anyhow::bail!(
            "unsupported network whitelist network id: {network_id}; expected mainnet or testnet11"
        );
    }

    Ok(())
}

pub fn build_user_grantable_capability_set(
    requested: &SageRequestedCapabilities,
    capabilities: impl IntoIterator<Item = UserBridgeCapability>,
) -> anyhow::Result<BTreeSet<UserBridgeCapability>> {
    let capabilities = capabilities.into_iter().collect::<BTreeSet<_>>();

    validate_user_granted_capabilities(requested, &capabilities)?;
    validate_required_user_grantable_capabilities_present(requested, &capabilities)?;

    Ok(capabilities)
}

pub fn validate_user_granted_capabilities(
    requested: &SageRequestedCapabilities,
    user_granted: &BTreeSet<UserBridgeCapability>,
) -> anyhow::Result<()> {
    for capability in user_granted {
        let definition = get_user_capability_definition(*capability);
        let flags = definition.flags();

        if !flags.user_grantable() {
            anyhow::bail!(
                "granted capability is not user grantable: {}",
                capability.key()
            );
        }

        if flags.requestable_by_app() && !requested.is_allowed(*capability) {
            anyhow::bail!(
                "granted capability not requested in manifest: {}",
                capability.key()
            );
        }
    }

    Ok(())
}

pub fn validate_required_user_grantable_capabilities_present(
    requested: &SageRequestedCapabilities,
    user_granted: &BTreeSet<UserBridgeCapability>,
) -> anyhow::Result<()> {
    for capability in requested.required() {
        let definition = get_user_capability_definition(*capability);

        if definition.flags().user_grantable() && !user_granted.contains(capability) {
            anyhow::bail!("missing required capability: {}", capability.key());
        }
    }

    Ok(())
}

pub fn split_required_optional_set<T: Ord>(
    required: impl IntoIterator<Item = T>,
    optional: impl IntoIterator<Item = T>,
) -> (BTreeSet<T>, BTreeSet<T>) {
    let required = required.into_iter().collect::<BTreeSet<_>>();

    let optional = optional
        .into_iter()
        .filter(|item| !required.contains(item))
        .collect::<BTreeSet<_>>();

    (required, optional)
}
