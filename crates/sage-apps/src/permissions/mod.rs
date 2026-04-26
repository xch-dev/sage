mod validation;
mod capabilities;
mod network;
mod normalization;

pub(crate) use capabilities::{
    get_user_capability_definition,
    normalize_and_validate_user_granted_capabilities,
    resolve_and_validate_effective_granted_capabilities,
    requested_user_grantable_capabilities,
    user_capability_definition_view,
    user_registry,
    get_system_capability_definition,
    CapabilityFlags,
};
pub(crate) use network::normalize_and_validate_granted_network;

use anyhow::Result;
use crate::permissions::normalization::normalize_requested_permissions;
use crate::permissions::validation::validate_requested_permission;
use crate::types::{SageGrantedNetworkPermissions, SageGrantedPermissions, SageRequestedPermissions};

pub(crate) fn normalize_and_validate_requested_permissions(
    permissions: &SageRequestedPermissions,
) -> Result<SageRequestedPermissions> {
    let normalized_requested = normalize_requested_permissions(permissions)?;

    validate_requested_permission(&normalized_requested)?;

    Ok(normalized_requested)
}

pub(crate) fn normalize_and_validate_granted_permissions(
    requested: &SageRequestedPermissions,
    granted: SageGrantedPermissions,
) -> Result<SageGrantedPermissions> {
    Ok(SageGrantedPermissions {
        capabilities: normalize_and_validate_user_granted_capabilities(
            &requested.capabilities,
            &granted.capabilities
        )?,
        network: SageGrantedNetworkPermissions {
            whitelist: normalize_and_validate_granted_network(
                &requested.network,
                &granted.network.whitelist,
            )?
        },
    })
}

#[cfg(test)]
pub(super) mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::permissions::{normalize_and_validate_requested_permissions};
    use crate::permissions::capabilities::user_registry;
    use crate::types::{SageNetworkPermissionTarget, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedNetworkWhitelist, SageRequestedPermissions};

    pub fn auto_granted_capability() -> UserBridgeCapability {
        UserBridgeCapability::AppGetInfo
    }

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
}
