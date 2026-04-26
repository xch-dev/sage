use anyhow::{anyhow, Result};
use crate::permissions::capabilities::CapabilityFlags;
use crate::types::{SageRequestedPermissions};

pub(super) fn validate_requested_permission(
    permissions: &SageRequestedPermissions,
) -> Result<()> {
    let mut requested = Vec::new();
    requested.extend(permissions.capabilities.required.iter().copied());
    requested.extend(permissions.capabilities.optional.iter().copied());


    let requested_capability_flags = CapabilityFlags::from_capabilities(&requested);

    if requested_capability_flags.externally_observable && requested_capability_flags.accesses_sensitive_secret {
        return Err(anyhow!(
            "requested permissions cannot include both externally observable and sensitive secret access permissions"
        ));
    }

    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::permissions::tests::tests::empty_requested_permissions;
    use crate::permissions::validate_requested_permission;

    #[test]
    fn requested_permissions_policy_rejects_secret_and_external_combination() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![
            UserBridgeCapability::WalletSendXch,
            UserBridgeCapability::WalletGetSecretKey,
        ];

        let err = validate_requested_permission(&requested)
            .expect_err("expected incompatible requested capability policy to be rejected");

        assert!(
            err.to_string().contains("requested permissions cannot include both externally observable and sensitive secret access permissions"),
            "unexpected error: {err}"
        );
    }
}
