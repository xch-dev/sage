use anyhow::{anyhow, Result as AnyResult};
use crate::bridge::capabilities::UserBridgeCapability;
use crate::{
    types::{SageAppCapabilityFlags},
};
use crate::permissions::capabilities::types::CapabilityFlags;

pub fn resolve_capability_flags(
    granted: &[UserBridgeCapability],
    previous_flags: Option<&SageAppCapabilityFlags>,
) -> AnyResult<SageAppCapabilityFlags> {
    let granted_capability_flags = CapabilityFlags::from_capabilities(granted)?;

    let previous_storage_may_contain_secrets = previous_flags
        .map(|flags| flags.storage_may_contain_secrets)
        .unwrap_or(false);

    let has_secret_access = granted_capability_flags.accesses_sensitive_secret;
    let has_external_access = granted_capability_flags.externally_observable;
    let storage_may_contain_secrets = previous_storage_may_contain_secrets;

    if has_external_access && has_secret_access {
        return Err(anyhow!(
            "cannot grant externally observable permissions together with sensitive secret access permissions"
        ));
    }

    if has_external_access && storage_may_contain_secrets {
        return Err(anyhow!("STORAGE_TAINTED"));
    }

    Ok(SageAppCapabilityFlags {
        has_secret_access,
        has_external_access,
        storage_may_contain_secrets,
        isolated: has_secret_access || storage_may_contain_secrets,
    })
}

pub fn mark_storage_may_contain_secrets(
    flags: &SageAppCapabilityFlags,
) -> SageAppCapabilityFlags {
    SageAppCapabilityFlags {
        has_secret_access: flags.has_secret_access,
        has_external_access: flags.has_external_access,
        storage_may_contain_secrets: true,
        isolated: true,
    }
}

pub fn clear_storage_may_contain_secrets(
    flags: &SageAppCapabilityFlags,
) -> SageAppCapabilityFlags {
    SageAppCapabilityFlags {
        has_secret_access: flags.has_secret_access,
        has_external_access: flags.has_external_access,
        storage_may_contain_secrets: false,
        isolated: flags.has_secret_access,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{normalize_and_validate_requested_permissions, validate_requested_permission};
    use crate::permissions::capabilities::definitions::user_registry;
    use crate::permissions::capabilities::get_effective_granted_capabilities;
    use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
    use crate::permissions::capabilities::validation::validate_granted_capabilities;
    use crate::types::{SageNetworkPermissionTarget, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions};

    fn empty_requested_permissions() -> SageRequestedPermissions {
        SageRequestedPermissions {
            network: SageRequestedNetworkPermissions {
                whitelist: SageRequestedNetworkWhitelist {
                    required: vec![],
                    optional: vec![],
                },
            },
            capabilities: SageRequestedCapabilities {
                required: vec![],
                optional: vec![],
            },
        }
    }

    fn first_non_requestable_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| !definition.flags.requestable_by_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with requestable_by_app = false")
            })
            .capability
    }

    fn first_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| definition.flags.shared_with_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = true")
            })
            .capability
    }

    fn first_non_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| !definition.flags.shared_with_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = false")
            })
            .capability
    }

    #[test]
    fn rejects_non_requestable_required_capability() {
        let non_requestable = first_non_requestable_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![non_requestable.clone()];

        let err = normalize_and_validate_requested_permissions(&requested)
            .expect_err("expected non-requestable required capability to be rejected");

        let message = err.to_string();
        assert!(
            message.contains(&non_requestable.key()),
            "error should mention rejected capability, got: {message}"
        );
    }

    #[test]
    fn rejects_non_requestable_optional_capability() {
        let non_requestable = first_non_requestable_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.optional = vec![non_requestable.clone()];

        let err = normalize_and_validate_requested_permissions(&requested)
            .expect_err("expected non-requestable optional capability to be rejected");

        let message = err.to_string();
        assert!(
            message.contains(&non_requestable.key()),
            "error should mention rejected capability, got: {message}"
        );
    }

    #[test]
    fn resolve_shared_capabilities_filters_out_non_shared_capabilities() {
        let shared = first_shared_capability();
        let non_shared = first_non_shared_capability();

        let shared_capabilities = UserBridgeCapability::shared_from_granted(&vec![
            shared.clone(),
            non_shared.clone(),
        ]);

        assert!(
            shared_capabilities.contains(&shared),
            "shared capability should remain visible to app"
        );
        assert!(
            !shared_capabilities.contains(&non_shared),
            "non-shared capability should not be visible to app"
        );
    }

    #[test]
    fn resolve_shared_capabilities_preserves_ordered_unique_shared_subset() {
        let shared = first_shared_capability();
        let non_shared = first_non_shared_capability();

        let shared_capabilities = UserBridgeCapability::shared_from_granted(&vec![
            non_shared.clone(),
            shared.clone(),
            shared.clone(),
        ]);

        assert_eq!(shared_capabilities, vec![shared]);
    }

    #[test]
    fn normalize_requested_permissions_deduplicates_and_sorts_capabilities() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![
            UserBridgeCapability::WalletSendXch,
            UserBridgeCapability::WalletSendXch,
        ];
        requested.capabilities.optional = vec![
            UserBridgeCapability::WalletSendXch,
        ];

        let normalized = normalize_and_validate_requested_permissions(&requested)
            .expect("expected requested permissions to normalize");

        assert_eq!(
            normalized.capabilities.required,
            vec![UserBridgeCapability::WalletSendXch]
        );
        assert!(normalized.capabilities.optional.is_empty());
    }

    #[test]
    fn normalize_requested_permissions_deduplicates_and_sorts_network_entries() {
        let mut requested = empty_requested_permissions();
        requested.network.whitelist.required = vec![
            SageNetworkPermissionTarget {
                scheme: "HTTPS".to_string(),
                host: "Example.com".to_string(),
            },
            SageNetworkPermissionTarget {
                scheme: "https".to_string(),
                host: "example.com".to_string(),
            },
        ];
        requested.network.whitelist.optional = vec![
            SageNetworkPermissionTarget {
                scheme: "WSS".to_string(),
                host: "ws.example.com".to_string(),
            },
            SageNetworkPermissionTarget {
                scheme: "https".to_string(),
                host: "example.com".to_string(),
            },
        ];

        let normalized = normalize_and_validate_requested_permissions(&requested)
            .expect("expected requested permissions to normalize");

        assert_eq!(
            normalized.network.whitelist.required,
            vec![SageNetworkPermissionTarget {
                scheme: "https".to_string(),
                host: "example.com".to_string(),
            }]
        );

        assert_eq!(
            normalized.network.whitelist.optional,
            vec![SageNetworkPermissionTarget {
                scheme: "wss".to_string(),
                host: "ws.example.com".to_string(),
            }]
        );
    }

    #[test]
    fn validate_granted_capabilities_rejects_unrequested_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_granted_capabilities(
            &requested,
            &vec![
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
    fn validate_granted_capabilities_rejects_missing_required_capability() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_granted_capabilities(&requested, &[])
            .expect_err("expected missing required capability to be rejected");

        assert!(
            err.to_string().contains(UserBridgeCapability::WalletSendXch.key()),
            "error should mention missing required capability"
        );
    }

    #[test]
    fn validate_granted_capabilities_allows_subset_of_optional_capabilities() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];
        requested.capabilities.optional = vec![UserBridgeCapability::PersistentStorage];

        validate_granted_capabilities(
            &requested,
            &vec![UserBridgeCapability::WalletSendXch],
        )
            .expect("expected optional capability to be omittable");
    }

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

    #[test]
    fn resolve_capability_flags_sets_expected_flags_for_shared_send_capability() {
        let flags = resolve_capability_flags(&vec![UserBridgeCapability::WalletSendXch], None)
            .expect("expected capability flags to resolve");

        assert!(flags.has_external_access);
        assert!(!flags.has_secret_access);
        assert!(!flags.storage_may_contain_secrets);
        assert!(!flags.isolated);
    }

    #[test]
    fn resolve_capability_flags_rejects_external_access_when_storage_is_tainted() {
        let previous = SageAppCapabilityFlags {
            has_secret_access: false,
            has_external_access: false,
            storage_may_contain_secrets: true,
            isolated: true,
        };

        let err = resolve_capability_flags(
            &vec![UserBridgeCapability::WalletSendXch],
            Some(&previous),
        )
            .expect_err("expected tainted storage to block externally observable capability");

        assert_eq!(err.to_string(), "STORAGE_TAINTED");
    }

    #[test]
    fn mark_storage_may_contain_secrets_sets_taint_and_isolation() {
        let flags = SageAppCapabilityFlags {
            has_secret_access: true,
            has_external_access: false,
            storage_may_contain_secrets: false,
            isolated: true,
        };

        let updated = mark_storage_may_contain_secrets(&flags);
        assert!(updated.storage_may_contain_secrets);
        assert!(updated.isolated);
        assert!(updated.has_secret_access);
        assert!(!updated.has_external_access);
    }

    #[test]
    fn clear_storage_may_contain_secrets_preserves_secret_access_isolation_only() {
        let flags = SageAppCapabilityFlags {
            has_secret_access: true,
            has_external_access: false,
            storage_may_contain_secrets: true,
            isolated: true,
        };

        let updated = clear_storage_may_contain_secrets(&flags);
        assert!(!updated.storage_may_contain_secrets);
        assert!(updated.isolated);
        assert!(updated.has_secret_access);
    }

    fn auto_granted_capability() -> UserBridgeCapability {
        UserBridgeCapability::AppGetInfo
    }

    #[test]
    fn non_user_grantable_required_capability_is_effective_without_persisted_grant() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        validate_granted_capabilities(&requested, &[])
            .expect("non-user-grantable required capability should not require persisted user grant");

        let effective = get_effective_granted_capabilities(&requested, &[])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn non_user_grantable_optional_capability_is_effective_without_persisted_grant() {
        let auto = auto_granted_capability();

        let mut requested = empty_requested_permissions();
        requested.capabilities.optional = vec![auto];

        validate_granted_capabilities(&requested, &[])
            .expect("non-user-grantable optional capability should not require persisted user grant");

        let effective = get_effective_granted_capabilities(&requested, &[])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn normalize_user_granted_capabilities_strips_non_user_grantable_capability() {
        let auto = auto_granted_capability();

        let normalized = normalize_granted_capabilities(&[auto])
            .expect("normalization should tolerate and strip stale non-user-grantable grants");

        assert!(normalized.is_empty());

        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![auto];

        let effective = get_effective_granted_capabilities(&requested, &normalized)
            .expect("auto capability should still be effective");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn moving_non_user_grantable_capability_from_optional_to_required_still_auto_grants() {
        let auto = auto_granted_capability();

        let mut optional_requested = empty_requested_permissions();
        optional_requested.capabilities.optional = vec![auto];

        let optional_effective = get_effective_granted_capabilities(
            &optional_requested,
            &[],
        )
            .expect("optional auto grant should resolve");

        assert_eq!(optional_effective, vec![auto]);

        let mut required_requested = empty_requested_permissions();
        required_requested.capabilities.required = vec![auto];

        let required_effective = get_effective_granted_capabilities(
            &required_requested,
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

        let effective = get_effective_granted_capabilities(&requested, &[])
            .expect("expected auto grant before removal");

        assert_eq!(effective, vec![auto]);

        let removed_requested = empty_requested_permissions();

        let effective_after_removal =
            get_effective_granted_capabilities(&removed_requested, &[])
                .expect("expected permissions to resolve after removal");

        assert!(effective_after_removal.is_empty());
    }

    #[test]
    fn user_grantable_required_capability_without_user_grant_is_blocked() {
        let mut requested = empty_requested_permissions();
        requested.capabilities.required = vec![UserBridgeCapability::WalletSendXch];

        let err = validate_granted_capabilities(&requested, &[])
            .expect_err("user-grantable required capability should require user grant");

        assert!(
            err.to_string().contains(UserBridgeCapability::WalletSendXch.key()),
            "error should mention missing user-grantable required capability"
        );

        get_effective_granted_capabilities(&requested, &[])
            .expect_err("effective permissions should not resolve without required user grant");
    }
}
