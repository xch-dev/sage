use thiserror::Error;

use crate::{Capability, Identity, endpoint_capability};

/// The outcome of a successful check; callers must treat any variant other than `Allowed` as not allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[must_use]
pub enum Decision {
    Allowed,
}

/// Why a check refused a request.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthError {
    #[error("permission denied: {what}")]
    PermissionDenied { what: String },
    #[error("no wallet is logged in")]
    NoWallet,
    #[error("the logged in wallet is not in scope")]
    WalletNotInScope,
}

impl AuthError {
    /// A stable machine-readable code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PermissionDenied { .. } => "permission_denied",
            Self::NoWallet => "no_wallet",
            Self::WalletNotInScope => "wallet_not_in_scope",
        }
    }
}

/// Checks that the identity holds the capability and may use the current wallet.
pub fn authorize(
    identity: &Identity,
    capability: Capability,
    current_fingerprint: Option<u32>,
) -> Result<Decision, AuthError> {
    let grant = match identity {
        Identity::Owner => return Ok(Decision::Allowed),
        Identity::Grantee(grant) => grant,
    };

    if !grant.capabilities.contains(&capability) {
        return Err(AuthError::PermissionDenied {
            what: capability.key().to_string(),
        });
    }

    let fingerprint = current_fingerprint.ok_or(AuthError::NoWallet)?;
    if !grant.wallet_scope.allows(fingerprint) {
        return Err(AuthError::WalletNotInScope);
    }

    Ok(Decision::Allowed)
}

/// Like [`authorize`], for a sage-api endpoint; grantees can only reach mapped endpoints.
pub fn authorize_endpoint(
    identity: &Identity,
    endpoint: &str,
    current_fingerprint: Option<u32>,
) -> Result<Decision, AuthError> {
    if matches!(identity, Identity::Owner) {
        return Ok(Decision::Allowed);
    }

    let capability = endpoint_capability(endpoint).ok_or_else(|| AuthError::PermissionDenied {
        what: endpoint.to_string(),
    })?;

    authorize(identity, capability, current_fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Agent, Grant, WalletScope};
    use Capability::*;

    fn grantee(capabilities: &[Capability], wallet_scope: WalletScope) -> Identity {
        Identity::Grantee(Grant {
            capabilities: capabilities.iter().copied().collect(),
            wallet_scope,
        })
    }

    #[test]
    fn owner_is_allowed_everything() {
        assert!(authorize(&Identity::Owner, WalletGetCoins, None).is_ok());
        for e in ["get_coins", "send_xch", "not_an_endpoint"] {
            assert!(authorize_endpoint(&Identity::Owner, e, None).is_ok(), "{e}");
        }
    }

    #[test]
    fn grantee_on_unmapped_endpoint_is_denied() {
        let g = grantee(Capability::ALL, WalletScope::AllWallets);
        assert_eq!(
            authorize_endpoint(&g, "send_xch", Some(1))
                .unwrap_err()
                .code(),
            "permission_denied"
        );
    }

    #[test]
    fn grantee_without_the_capability_is_denied() {
        let g = grantee(&[WalletGetVersion], WalletScope::AllWallets);
        assert_eq!(
            authorize(&g, WalletGetCoins, Some(1)).unwrap_err().code(),
            "permission_denied"
        );
        assert_eq!(
            authorize_endpoint(&g, "get_coins", Some(1))
                .unwrap_err()
                .code(),
            "permission_denied"
        );
    }

    #[test]
    fn grantee_with_no_wallet_logged_in_is_refused() {
        let g = grantee(&[WalletGetCoins], WalletScope::AllWallets);
        assert_eq!(
            authorize(&g, WalletGetCoins, None).unwrap_err().code(),
            "no_wallet"
        );
    }

    #[test]
    fn grantee_outside_its_scope_is_refused() {
        let g = grantee(
            &[WalletGetCoins],
            WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        );
        assert_eq!(
            authorize(&g, WalletGetCoins, Some(8)).unwrap_err().code(),
            "wallet_not_in_scope"
        );
        assert!(authorize(&g, WalletGetCoins, Some(7)).is_ok());
        assert!(authorize_endpoint(&g, "get_coins", Some(7)).is_ok());
    }

    #[test]
    fn capability_is_checked_before_wallet() {
        let g = grantee(&[], WalletScope::AllWallets);
        assert_eq!(
            authorize(&g, WalletGetCoins, None).unwrap_err().code(),
            "permission_denied"
        );
    }

    #[test]
    fn default_scope_allows_nothing() {
        assert!(!WalletScope::default().allows(1));
    }

    #[test]
    fn agent_identity_carries_its_grant() {
        let a = Agent {
            id: "a".into(),
            name: "n".into(),
            capabilities: [WalletGetCoins].into(),
            wallet_scope: WalletScope::AllWallets,
            created_at_ms: 0,
            updated_at_ms: 0,
            last_used_at_ms: None,
        };
        assert!(authorize(&a.identity(), WalletGetCoins, Some(1)).is_ok());
        assert!(authorize(&a.identity(), WalletGetVersion, Some(1)).is_err());
    }
}
