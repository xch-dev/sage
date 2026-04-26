use std::collections::BTreeSet;
use anyhow::{anyhow, Result};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::permissions::capabilities::definitions::get_user_capability_definition;
use crate::types::{SageRequestedCapabilities};

pub(in crate::permissions) fn validate_effective_granted_capabilities(
    requested: &SageRequestedCapabilities,
    effective: &[UserBridgeCapability],
) -> Result<()> {
    validate_requested_capability_subset(requested, effective, "effective")
}

pub(in crate::permissions) fn validate_user_granted_capabilities(
    requested: &SageRequestedCapabilities,
    granted: &[UserBridgeCapability],
) -> Result<()> {
    validate_requested_capability_subset(requested, granted, "granted")?;

    let granted_set: BTreeSet<_> = granted.iter().copied().collect();

    for required_capability in &requested.required {
        let definition = get_user_capability_definition(*required_capability);

        if definition.flags.user_grantable && !granted_set.contains(required_capability) {
            return Err(anyhow!(
                "missing required capability: {}",
                required_capability.key()
            ));
        }
    }

    Ok(())
}

fn validate_requested_capability_subset(
    requested: &SageRequestedCapabilities,
    capabilities: &[UserBridgeCapability],
    label: &str,
) -> Result<()> {
    let allowed: BTreeSet<_> = requested
        .required
        .iter()
        .chain(requested.optional.iter())
        .copied()
        .collect();

    for capability in capabilities {
        if !allowed.contains(capability) {
            return Err(anyhow!(
                "{label} capability not requested in manifest: {}",
                capability.key()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::permissions::capabilities::validation::{validate_effective_granted_capabilities, validate_user_granted_capabilities};
    use crate::permissions::resolve_and_validate_effective_granted_capabilities;
    use crate::permissions::tests::{auto_granted_capability, empty_requested_permissions};

    #[test]
    fn validate_user_granted_capabilities_rejects_unrequested_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_user_granted_capabilities(
            &requested.capabilities,
            &[
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
        )
            .expect_err("expected unrequested capability to be rejected");

        assert!(
            err.to_string().contains("persistent_storage"),
            "error should mention unrequested capability"
        );
    }

    #[test]
    fn validate_user_granted_capabilities_rejects_missing_required_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_user_granted_capabilities(&requested.capabilities, &[])
            .expect_err("expected missing required capability to be rejected");

        assert!(
            err.to_string().contains(UserBridgeCapability::WalletSendXch.key()),
            "error should mention missing required capability"
        );
    }

    #[test]
    fn validate_user_granted_capabilities_allows_subset_of_optional_capabilities() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];
        requested.capabilities.optional = vec![UserBridgeCapability::PersistentStorage];

        validate_user_granted_capabilities(
            &requested.capabilities,
            &[UserBridgeCapability::WalletSendXch],
        )
            .expect("expected optional capability to be omittable");
    }


    #[test]
    fn non_user_grantable_required_capability_is_effective_without_persisted_grant() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        validate_user_granted_capabilities(&requested.capabilities, &[])
            .expect("non-user-grantable required capability should not require persisted user grant");

        let effective = resolve_and_validate_effective_granted_capabilities(&requested.capabilities, &[])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn non_user_grantable_optional_capability_is_effective_without_persisted_grant() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.optional = vec![auto];

        validate_user_granted_capabilities(&requested.capabilities, &[])
            .expect("non-user-grantable optional capability should not require persisted user grant");

        let effective = resolve_and_validate_effective_granted_capabilities(&requested.capabilities, &[])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn user_grantable_required_capability_without_user_grant_is_blocked() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_user_granted_capabilities(&requested.capabilities, &[])
            .expect_err("user-grantable required capability should require user grant");

        assert!(
            err.to_string().contains(UserBridgeCapability::WalletSendXch.key()),
            "error should mention missing user-grantable required capability"
        );

        resolve_and_validate_effective_granted_capabilities(&requested.capabilities, &[])
            .expect_err("effective permissions should not resolve without required user grant");
    }

    #[test]
    fn validate_effective_granted_capabilities_rejects_unrequested_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_effective_granted_capabilities(
            &requested.capabilities,
            &[
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
        )
            .expect_err("expected unrequested effective capability to be rejected");

        assert!(
            err.to_string().contains("persistent_storage"),
            "error should mention unrequested capability"
        );
    }

    #[test]
    fn validate_effective_granted_capabilities_allows_user_grantable_required_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        validate_effective_granted_capabilities(
            &requested.capabilities,
            &[UserBridgeCapability::WalletSendXch],
        )
            .expect("expected requested required capability to be valid as effective");
    }

    #[test]
    fn validate_effective_granted_capabilities_allows_non_user_grantable_required_capability() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        validate_effective_granted_capabilities(&requested.capabilities, &[auto])
            .expect("expected requested non-user-grantable capability to be valid as effective");
    }

    #[test]
    fn validate_effective_granted_capabilities_allows_requested_optional_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.optional = vec![UserBridgeCapability::PersistentStorage];

        validate_effective_granted_capabilities(
            &requested.capabilities,
            &[UserBridgeCapability::PersistentStorage],
        )
            .expect("expected requested optional capability to be valid as effective");
    }

    #[test]
    fn validate_effective_granted_capabilities_allows_empty_effective_set() {
        let requested = empty_requested_permissions();

        validate_effective_granted_capabilities(&requested.capabilities, &[])
            .expect("empty effective capabilities should be valid when nothing is effective");
    }
}
