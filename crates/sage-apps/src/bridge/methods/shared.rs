use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{
    AppState, AppsHostState, BridgeCapability, RustBridgeApprovalRequest,
    RustBridgeApprovalResponse, RustBridgeRequest, SharedSageApp, SystemBridgeCapability,
    UserBridgeCapability,
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

#[derive(Debug)]
pub(crate) struct BridgeContext<'a> {
    pub app: &'a SharedSageApp,
}

#[derive(Debug)]
pub(crate) struct BridgeTools<'a> {
    pub app_handle: &'a tauri::AppHandle,
    pub app_state: &'a tauri::State<'a, AppState>,
    pub caller: &'a SharedSageApp,
    pub invocation: &'a RustBridgeRequest,
    pub approval: Option<&'a sage_permissions::ApprovalBinding>,
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

    pub(super) fn user(cap: UserBridgeCapability) -> Self {
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

impl BridgeTools<'_> {
    pub async fn approved_wallet(
        &self,
        method: &str,
        request: &impl serde::Serialize,
    ) -> Result<sage_rpc::RpcSession<'_>, BridgeMethodHandleError> {
        let mut session = self.wallet().await?;
        session
            .bind_approved_request(method, request)
            .map_err(|err| BridgeMethodHandleError::new("approval_invalid", err.to_string()))?;
        Ok(session)
    }

    /// Resolve current grants only after acquiring the wallet lock, and keep that lock
    /// through RPC authorization and execution.
    pub async fn wallet(&self) -> Result<sage_rpc::RpcSession<'_>, BridgeMethodHandleError> {
        let sage = self.app_state.lock().await;
        let caller_id = self.caller.id();
        let caller = self.caller.with(|app| sage_permissions::Caller {
            id: sage_permissions::CallerId::App(caller_id),
            capabilities: app
                .common()
                .requested_permissions()
                .capabilities()
                .resolve_effective_grants(
                    app.common().granted_permissions().capabilities().copied(),
                )
                .into_iter()
                .filter_map(UserBridgeCapability::wallet_capability)
                .collect(),
            wallet_scope: app.common().wallet_scope().clone(),
        });
        caller
            .require_wallet(sage.config.global.fingerprint)
            .map_err(|err| BridgeMethodHandleError::new("permission_denied", err.to_string()))?;
        let invocation = serde_json::to_string(self.invocation)
            .map_err(|err| BridgeMethodHandleError::internal_error(err.to_string()))?;
        Ok(sage_rpc::RpcSession::scoped(
            sage,
            caller,
            invocation,
            self.approval.cloned(),
        ))
    }
}
