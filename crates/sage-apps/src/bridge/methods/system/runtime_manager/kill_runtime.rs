use async_trait::async_trait;
use serde::Serialize;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools, kill_runtime, parse_required_params, RuntimeTargetParams, RustBridgeRequest, SystemBridgeCapability, SystemKillRuntimeError};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerKillRuntime;

#[derive(Debug, Copy, Clone, Serialize)]
pub struct RuntimeManagerKillRuntimeResponse {
    ok: bool,
}

impl RuntimeManagerKillRuntimeResponse {
    pub fn ok() -> Self {
        Self { ok: true }
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
            Ok(())
            | Err(SystemKillRuntimeError::NotFound | SystemKillRuntimeError::RuntimeSync(_)) => {
                Ok(Box::new(RuntimeManagerKillRuntimeResponse::ok()))
            }
        }
    }
}
