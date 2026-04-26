#[cfg(test)]
pub(super) mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::flags::{clear_storage_may_contain_secrets, get_app_flags, mark_storage_may_contain_secrets};
    use crate::permissions::{normalize_and_validate_requested_permissions};
    use crate::permissions::capabilities::user_registry;
    use crate::permissions::capabilities::resolve_and_validate_effective_granted_capabilities;
    use crate::types::{SageAppFlags, SageNetworkPermissionTarget, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions};

    pub fn empty_requested_permissions() -> SageRequestedPermissions {
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
    fn resolve_capability_flags_sets_expected_flags_for_shared_send_capability() {
        let flags = get_app_flags(&vec![UserBridgeCapability::WalletSendXch], None)
            .expect("expected capability flags to resolve");

        assert!(flags.has_external_access);
        assert!(!flags.has_secret_access);
        assert!(!flags.storage_may_contain_secrets);
        assert!(!flags.isolated);
    }

    #[test]
    fn resolve_capability_flags_rejects_external_access_when_storage_is_tainted() {
        let previous = SageAppFlags {
            has_secret_access: false,
            has_external_access: false,
            storage_may_contain_secrets: true,
            isolated: true,
        };

        let err = get_app_flags(
            &vec![UserBridgeCapability::WalletSendXch],
            Some(&previous),
        )
            .expect_err("expected tainted storage to block externally observable capability");

        assert_eq!(err.to_string(), "STORAGE_TAINTED");
    }

    #[test]
    fn mark_storage_may_contain_secrets_sets_taint_and_isolation() {
        let flags = SageAppFlags {
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
        let flags = SageAppFlags {
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

    pub fn auto_granted_capability() -> UserBridgeCapability {
        UserBridgeCapability::AppGetInfo
    }

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
