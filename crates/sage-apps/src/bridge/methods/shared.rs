use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{AppsHostState, AppState, BridgeCapability, RustBridgeApprovalRequest, RustBridgeRequest, SharedSageApp, SystemBridgeCapability, UserBridgeCapability};

#[async_trait]
pub(crate) trait BridgeMethod: Send + Sync {
    fn name(&self) -> &'static str;
    fn capability(&self) -> BridgeMethodCapability;

    fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult;

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
