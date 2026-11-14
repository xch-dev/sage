use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{
    AppState, AppsHostState, BridgeCapability, RustBridgeApprovalRequest,
    RustBridgeApprovalResponse, RustBridgeRequest, SageAgent, SharedSageApp,
    SystemBridgeCapability, UserBridgeCapability,
};

#[async_trait]
pub(crate) trait BridgeMethod: Send + Sync {
    fn name(&self) -> &'static str;
    fn capability(&self) -> BridgeMethodCapability;

    fn deprecated_in_favor_of(&self) -> Option<&'static str> {
        None
    }

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let _ = (ctx, request);
        Ok(None)
    }

    async fn prepare_approval(
        &self,
        ctx: BridgeContext<'_>,
        _tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        self.approval_request(ctx, request)
    }

    fn binds_approval_to_wallet(&self) -> bool {
        false
    }

    /// Whether the method signs with the wallet's keys. Watch wallets have none, so these are
    /// refused before the user is asked to approve them.
    fn requires_signing(&self) -> bool {
        false
    }

    /// Override only when approved execution differs from the normal request path.
    async fn handle_approved(
        &self,
        _approval: &RustBridgeApprovalRequest,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
        response: Option<&RustBridgeApprovalResponse>,
    ) -> BridgeHandleResult {
        if response.is_some() {
            return Err(BridgeMethodHandleError::invalid_request(format!(
                "{} does not accept approval response data",
                self.name()
            )));
        }

        self.handle(ctx, tools, request).await
    }

    async fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BridgeMethodCapability {
    Ungated,
    Required(BridgeCapability),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BridgeCaller<'a> {
    App(&'a SharedSageApp),
    Agent(&'a SageAgent),
}

#[derive(Debug)]
pub(crate) struct BridgeContext<'a> {
    pub caller: BridgeCaller<'a>,
}

impl BridgeContext<'_> {
    pub fn app(&self) -> Result<&SharedSageApp, BridgeMethodHandleError> {
        match self.caller {
            BridgeCaller::App(app) => Ok(app),
            BridgeCaller::Agent(_) => Err(BridgeMethodHandleError::new(
                "app_only",
                "This method is only available to Sage apps",
            )),
        }
    }

    pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool {
        match self.caller {
            BridgeCaller::App(app) => app.is_wallet_in_scope(fingerprint),
            BridgeCaller::Agent(agent) => agent.is_wallet_in_scope(fingerprint),
        }
    }
}

#[derive(Debug)]
pub(crate) struct BridgeTools<'a> {
    pub app_handle: &'a tauri::AppHandle,
    pub app_state: &'a tauri::State<'a, AppState>,
    pub host_state: &'a tauri::State<'a, AppsHostState>,
}

#[derive(Debug, Clone)]
pub(crate) struct BridgeMethodHandleError {
    pub code: &'static str,
    pub message: String,
}

pub(crate) type BridgeApprovalRequestResult =
    Result<Option<RustBridgeApprovalRequest>, BridgeMethodHandleError>;

impl BridgeMethodHandleError {
    pub(super) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(super) fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message)
    }

    pub(super) fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}

pub(crate) type BridgeHandleResult =
    Result<Box<dyn erased_serde::Serialize + Send>, BridgeMethodHandleError>;

impl BridgeMethodCapability {
    pub(super) fn ungated() -> Self {
        Self::Ungated
    }

    pub(crate) fn user(cap: UserBridgeCapability) -> Self {
        Self::Required(BridgeCapability::User(cap))
    }

    pub(super) fn system(cap: SystemBridgeCapability) -> Self {
        Self::Required(BridgeCapability::System(cap))
    }
}

pub(crate) fn parse_required_params<T>(
    method: &impl BridgeMethod,
    request: &RustBridgeRequest,
) -> Result<T, BridgeMethodHandleError>
where
    T: DeserializeOwned,
{
    let Some(params_json) = request.params_json.as_deref() else {
        return Err(BridgeMethodHandleError::new(
            "invalid_request",
            format!("{} requires params", method.name()),
        ));
    };

    serde_json::from_str(params_json).map_err(|err| {
        BridgeMethodHandleError::new(
            "invalid_request",
            format!("Failed to decode {} params: {err}", method.name()),
        )
    })
}

pub(crate) fn parse_optional_params<T>(
    method: &impl BridgeMethod,
    request: &RustBridgeRequest,
) -> Result<T, BridgeMethodHandleError>
where
    T: DeserializeOwned + Default,
{
    let Some(params_json) = request.params_json.as_deref() else {
        return Ok(T::default());
    };

    serde_json::from_str(params_json).map_err(|err| {
        BridgeMethodHandleError::new(
            "invalid_request",
            format!("Failed to decode {} params: {err}", method.name()),
        )
    })
}

#[cfg(test)]
mod bridge_caller_tests {
    use std::collections::BTreeSet;

    use crate::{SageAgent, SageAppWalletScope};

    use super::{BridgeCaller, BridgeContext};

    fn agent(wallet_scope: SageAppWalletScope) -> SageAgent {
        SageAgent {
            id: "agent".to_string(),
            name: "Agent".to_string(),
            capabilities: BTreeSet::new(),
            wallet_scope,
            created_at_ms: 0,
            updated_at_ms: 0,
            last_used_at_ms: None,
        }
    }

    #[test]
    fn agent_context_has_no_app() {
        let agent = agent(SageAppWalletScope::AllWallets);
        let ctx = BridgeContext {
            caller: BridgeCaller::Agent(&agent),
        };

        let err = ctx.app().unwrap_err();
        assert_eq!(err.code, "app_only");
    }

    #[test]
    fn agent_context_scope_uses_agent_scope() {
        let agent = agent(SageAppWalletScope::SelectedWallets {
            fingerprints: vec![7],
        });
        let ctx = BridgeContext {
            caller: BridgeCaller::Agent(&agent),
        };

        assert!(ctx.is_wallet_in_scope(7));
        assert!(!ctx.is_wallet_in_scope(8));
    }
}
