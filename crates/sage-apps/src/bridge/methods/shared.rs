use crate::AppsHostState;
use crate::bridge::capabilities::{BridgeCapability, SystemBridgeCapability, UserBridgeCapability};
use crate::bridge::{RustBridgeApprovalRequest, RustBridgeRequest};
use crate::host::AppState;
use crate::types::SharedSageApp;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use crate::runtime::SharedImpostorRuntime;

#[derive(Debug)]
pub struct BridgeContext<'a> {
    pub app: &'a SharedSageApp,
    pub impostor_runtime: &'a Option<SharedImpostorRuntime>,
}

#[derive(Debug)]
pub struct BridgeTools<'a> {
    pub app_handle: &'a tauri::AppHandle,
    pub app_state: &'a tauri::State<'a, AppState>,
    pub host_state: &'a tauri::State<'a, AppsHostState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMethodCapability {
    Ungated,
    Required(BridgeCapability),
}

#[derive(Debug, Clone)]
pub struct BridgeMethodHandleError {
    pub code: &'static str,
    pub message: String,
}

impl BridgeMethodHandleError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}

pub type BridgeApprovalRequestResult =
    Result<Option<RustBridgeApprovalRequest>, BridgeMethodHandleError>;

pub type BridgeHandleResult =
    Result<Box<dyn erased_serde::Serialize + Send>, BridgeMethodHandleError>;

#[async_trait]
pub trait BridgeMethod: Send + Sync {
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

impl BridgeMethodCapability {
    pub fn ungated() -> Self {
        Self::Ungated
    }

    pub fn user(cap: UserBridgeCapability) -> Self {
        Self::Required(BridgeCapability::User(cap))
    }

    pub fn system(cap: SystemBridgeCapability) -> Self {
        Self::Required(BridgeCapability::System(cap))
    }
}

pub fn parse_required_params<T>(
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
