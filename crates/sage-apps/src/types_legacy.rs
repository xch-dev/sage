use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::BTreeSet;
use uuid::Uuid;
use crate::bridge::capabilities::{
    SharedCapabilitiesExt, SystemBridgeCapability, UserBridgeCapability,
};
use crate::lifecycle::flags::get_app_flags;
use crate::lifecycle::{
    MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES, manifest_entry_file, manifest_icon_file,
    validate_manifest_file_path, validate_sha256_hex,
};
use crate::permissions::{CapabilityFlags, get_user_capability_definition};
use crate::sandbox::SANDBOX_TEST_ID_PREFIX;
use crate::utils::unix_timestamp_ms;



#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::permissions::user_registry;
    use crate::types::{
        SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities,
        SageRequestedNetworkPermissions, SageRequestedPermissions,
    };

    #[test]
    fn granted_permissions_rejects_non_user_grantable_capability_as_user_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([auto], []),
        )
        .expect("requested permissions should be valid");

        let err = SageGrantedPermissions::new(&requested, [auto], [])
            .expect_err("non-user-grantable capability cannot be persisted as user grant");

        assert!(
            err.to_string().contains("not user grantable"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn non_user_grantable_requested_capability_is_effective_without_user_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);

        let effective = requested
            .resolve_effective_grants([])
            .expect("auto capability should still be effective");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn effective_grants_include_non_user_grantable_requested_capability() {
        let auto = UserBridgeCapability::AppGetInfo;

        let optional_requested = SageRequestedCapabilities::new([], [auto]);
        assert_eq!(
            optional_requested.resolve_effective_grants([]).unwrap(),
            vec![auto]
        );

        let required_requested = SageRequestedCapabilities::new([auto], []);
        assert_eq!(
            required_requested.resolve_effective_grants([]).unwrap(),
            vec![auto]
        );
    }

    #[test]
    fn effective_grants_do_not_include_removed_non_user_grantable_capability() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);
        assert_eq!(requested.resolve_effective_grants([]).unwrap(), vec![auto]);

        let removed_requested = SageRequestedCapabilities::new([], []);
        assert!(
            removed_requested
                .resolve_effective_grants([])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_non_requestable_required_capability() {
        let non_requestable = first_non_requestable_capability();

        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([non_requestable], []),
        )
        .expect_err("expected non-requestable required capability to be rejected");

        let message = err.to_string();
        assert!(message.contains(non_requestable.key()));
    }

    #[test]
    fn rejects_non_requestable_optional_capability() {
        let non_requestable = first_non_requestable_capability();

        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([], [non_requestable]),
        )
        .expect_err("expected non-requestable optional capability to be rejected");

        let message = err.to_string();
        assert!(message.contains(non_requestable.key()));
    }

    #[test]
    fn requested_capabilities_deduplicates_and_removes_required_from_optional() {
        let requested = SageRequestedCapabilities::new(
            [
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::WalletSendXch,
            ],
            [UserBridgeCapability::WalletSendXch],
        );

        assert_eq!(
            requested.required().copied().collect::<Vec<_>>(),
            vec![UserBridgeCapability::WalletSendXch]
        );

        assert!(requested.optional().next().is_none());
    }

    #[test]
    fn requested_network_deduplicates_and_removes_required_from_optional() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [
                    SageNetworkWhitelistEntry::new("HTTPS", "Example.com").unwrap(),
                    SageNetworkWhitelistEntry::new("https", "example.com").unwrap(),
                ],
                [
                    SageNetworkWhitelistEntry::new("WSS", "ws.example.com").unwrap(),
                    SageNetworkWhitelistEntry::new("https", "example.com").unwrap(),
                ],
            ),
            SageRequestedCapabilities::empty(),
        )
        .unwrap();

        let required = requested
            .network
            .whitelist
            .required()
            .cloned()
            .collect::<Vec<_>>();

        let optional = requested
            .network
            .whitelist
            .optional()
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(
            required,
            vec![SageNetworkWhitelistEntry::new("https", "example.com").unwrap()]
        );

        assert_eq!(
            optional,
            vec![SageNetworkWhitelistEntry::new("wss", "ws.example.com").unwrap()]
        );
    }

    #[test]
    fn granted_permissions_rejects_unrequested_capability() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []),
        )
        .unwrap();

        let err = SageGrantedPermissions::new(
            &requested,
            [
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
            [],
        )
        .expect_err("expected unrequested capability to be rejected");

        assert!(err.to_string().contains("persistent_storage"));
    }

    #[test]
    fn granted_permissions_rejects_missing_required_user_grantable_capability() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []),
        )
        .unwrap();

        let err = SageGrantedPermissions::new(&requested, [], [])
            .expect_err("expected missing required capability to be rejected");

        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXch.key())
        );
    }

    #[test]
    fn granted_permissions_allows_subset_of_optional_capabilities() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::WalletSendXch],
                [UserBridgeCapability::PersistentStorage],
            ),
        )
        .unwrap();

        SageGrantedPermissions::new(&requested, [UserBridgeCapability::WalletSendXch], [])
            .expect("expected optional capability to be omittable");
    }

    #[test]
    fn non_user_grantable_required_capability_is_effective_without_persisted_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);

        let effective = requested
            .resolve_effective_grants([])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn non_user_grantable_optional_capability_is_effective_without_persisted_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([], [auto]);

        let effective = requested
            .resolve_effective_grants([])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn user_grantable_required_capability_without_user_grant_is_blocked() {
        let requested = SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []);

        let err = requested
            .resolve_effective_grants([])
            .expect_err("required user-grantable capability should require user grant");

        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXch.key())
        );
    }

    #[test]
    fn requested_permissions_policy_rejects_required_secret_and_external_combination() {
        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new(
                [
                    UserBridgeCapability::WalletSendXch,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
                [],
            ),
        )
        .expect_err("expected incompatible requested capability policy to be rejected");

        assert!(
            err.to_string().contains(
                "required requested permissions cannot include both external access and sensitive secret access"
            ),
            "unexpected error: {err}"
        );
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
}
