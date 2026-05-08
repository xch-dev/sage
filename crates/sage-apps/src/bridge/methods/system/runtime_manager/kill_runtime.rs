use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability, parse_required_params,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::runtime::RuntimeTargetParams;
use crate::runtime::stop::{SystemKillRuntimeError, kill_runtime};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerKillRuntime;

#[derive(Debug, Copy, Clone, Serialize)]
pub struct RuntimeManagerKillRuntimeResponse {
    ok: bool,
}

impl RuntimeManagerKillRuntimeResponse {
    pub fn ok() -> Box<Self> {
        Box::new(Self { ok: true })
    }
}

#[async_trait]
impl BridgeMethod for RuntimeManagerKillRuntime {
    fn name(&self) -> &'static str {
        "runtimeManager.killRuntime"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::RuntimeManagerKillRuntime)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: RuntimeTargetParams = parse_required_params(self, request)?;

        match kill_runtime(
            tools.app_handle,
            tools.host_state,
            &params.app_id,
            "user_kill",
        )
        .await
        {
            Ok(_)
            | Err(SystemKillRuntimeError::NotFound)
            | Err(SystemKillRuntimeError::RuntimeSync(_)) => {
                Ok(RuntimeManagerKillRuntimeResponse::ok())
            }
        }
    }
}
