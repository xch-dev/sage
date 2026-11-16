use std::collections::BTreeSet;
use std::fmt;

use crate::{WalletCapability, WalletScope, wallet_capability_definition};

/// Assigned by a trusted transport adapter, never accepted from request parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerId {
    App(String),
    Agent(String),
    Service(String),
}

/// Effective grants resolved by the host for one invocation. Resolve again on every request,
/// including after GUI approval, so revocation and scope edits take effect immediately.
#[derive(Debug, Clone)]
pub struct Caller {
    pub id: CallerId,
    pub capabilities: BTreeSet<WalletCapability>,
    pub wallet_scope: WalletScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionError(pub &'static str);

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for PermissionError {}

impl Caller {
    pub fn require(&self, capability: WalletCapability) -> Result<(), PermissionError> {
        validate_capability_policy(self.capabilities.iter().copied(), false)?;
        if !self.capabilities.contains(&capability) {
            return Err(PermissionError("Required capability has not been granted"));
        }
        Ok(())
    }

    pub fn require_wallet(&self, fingerprint: Option<u32>) -> Result<u32, PermissionError> {
        let fingerprint = fingerprint.ok_or(PermissionError("No wallet is selected"))?;
        if !self.wallet_scope.allows(fingerprint) {
            return Err(PermissionError("Wallet is outside the caller's scope"));
        }
        Ok(fingerprint)
    }
}

pub fn validate_capability_policy(
    capabilities: impl IntoIterator<Item = WalletCapability>,
    has_external_access: bool,
) -> Result<(), PermissionError> {
    let mut external = has_external_access;
    let mut secret = false;
    for capability in capabilities {
        let definition = wallet_capability_definition(capability);
        external |= definition.externally_observable;
        secret |= definition.accesses_sensitive_secret;
    }
    if secret && external {
        return Err(PermissionError(
            "Permissions cannot include both external access and sensitive secret access",
        ));
    }
    Ok(())
}

/// Shared GUI approval binding. The UI adapter owns presentation and response data; this
/// layer owns caller, request, wallet and expiry checks. It is deliberately not deserializable.
#[derive(Debug, Clone)]
pub struct ApprovalBinding {
    caller: CallerId,
    request: String,
    fingerprint: Option<u32>,
    expires_at_ms: u64,
}

impl ApprovalBinding {
    pub fn new(
        caller: CallerId,
        request: String,
        fingerprint: Option<u32>,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            caller,
            request,
            fingerprint,
            expires_at_ms,
        }
    }

    /// For a trusted GUI adapter translating the reviewed request to its final RPC payload
    /// (for example, applying the fee chosen in the approval UI). Keeps identity and expiry.
    #[must_use]
    pub fn with_request(&self, request: String) -> Self {
        Self {
            request,
            ..self.clone()
        }
    }

    pub fn validate_wallet(
        &self,
        caller: &CallerId,
        request: &str,
        fingerprint: u32,
        now_ms: u64,
    ) -> Result<(), PermissionError> {
        if self.fingerprint.is_none() {
            return Err(PermissionError("Approval is not bound to a wallet"));
        }
        self.validate(caller, request, Some(fingerprint), now_ms)
    }

    pub fn validate(
        &self,
        caller: &CallerId,
        request: &str,
        fingerprint: Option<u32>,
        now_ms: u64,
    ) -> Result<(), PermissionError> {
        if now_ms >= self.expires_at_ms {
            return Err(PermissionError("Approval has expired"));
        }
        if &self.caller != caller || self.request != request {
            return Err(PermissionError(
                "Approval belongs to a different caller or request",
            ));
        }
        if self.fingerprint.is_some() && self.fingerprint != fingerprint {
            return Err(PermissionError(
                "Active wallet changed since approval was requested",
            ));
        }
        Ok(())
    }
}

/// One-shot pending approval storage. Presentation and expiry notifications belong to the
/// host, while consuming an approval before execution prevents replay and duplicate clicks.
#[derive(Debug)]
pub struct ApprovalQueue<T> {
    pending: std::collections::BTreeMap<String, T>,
}

impl<T> Default for ApprovalQueue<T> {
    fn default() -> Self {
        Self {
            pending: std::collections::BTreeMap::new(),
        }
    }
}

impl<T> ApprovalQueue<T> {
    pub fn insert(&mut self, id: String, approval: T) -> Option<T> {
        self.pending.insert(id, approval)
    }
    pub fn remove(&mut self, id: &str) -> Option<T> {
        self.pending.remove(id)
    }
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.pending.values()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WalletCapability as Cap;

    fn caller(id: CallerId) -> Caller {
        Caller {
            id,
            capabilities: [Cap::WalletGetCoins].into(),
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        }
    }

    #[test]
    fn apps_and_agents_use_the_same_capabilities_and_scopes() {
        for id in [
            CallerId::App("reader".into()),
            CallerId::Agent("reader".into()),
        ] {
            let caller = caller(id);
            assert!(caller.require(Cap::WalletGetCoins).is_ok());
            assert!(caller.require(Cap::WalletGetSecretKey).is_err());
            assert_eq!(caller.require_wallet(Some(7)), Ok(7));
            assert!(caller.require_wallet(Some(8)).is_err());
            assert!(caller.require_wallet(None).is_err());
        }
        assert!(!WalletScope::default().allows(7));
        assert!(WalletScope::AllWallets.allows(7));
    }

    #[test]
    fn secret_and_external_permissions_cannot_be_combined() {
        assert!(validate_capability_policy([Cap::WalletGetSecretKey], false).is_ok());
        assert!(validate_capability_policy([Cap::WalletGetSecretKey], true).is_err());
        assert!(
            validate_capability_policy([Cap::WalletGetSecretKey, Cap::WalletSendXch], false)
                .is_err()
        );
    }

    #[test]
    fn approvals_bind_identity_request_wallet_and_expiry() {
        let id = CallerId::App("one".into());
        let binding = ApprovalBinding::new(id.clone(), "request".into(), Some(7), 100);
        assert!(binding.validate(&id, "request", Some(7), 99).is_ok());
        for (id, request, wallet, time) in [
            (CallerId::App("two".into()), "request", Some(7), 99),
            (CallerId::Agent("one".into()), "request", Some(7), 99),
            (id.clone(), "changed request", Some(7), 99),
            (id.clone(), "request", Some(8), 99),
            (id.clone(), "request", None, 99),
            (id, "request", Some(7), 100),
        ] {
            assert!(binding.validate(&id, request, wallet, time).is_err());
        }
    }

    #[test]
    fn an_approval_is_consumed_once() {
        let mut queue = ApprovalQueue::default();
        queue.insert("approval".into(), "request");
        assert_eq!(queue.remove("approval"), Some("request"));
        assert_eq!(queue.remove("approval"), None);
    }
}
