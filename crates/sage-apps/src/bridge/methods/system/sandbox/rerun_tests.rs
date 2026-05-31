use async_trait::async_trait;

use crate::{begin_sandbox_run, BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest, sandbox_runner, SystemBridgeCapability};

#[derive(Debug, Clone, Copy)]
pub struct SandboxRerunTests;

#[async_trait]
impl BridgeMethod for SandboxRerunTests {
    fn name(&self) -> &'static str {
        "sandbox.rerunTests"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::SandboxRerunTests)
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
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let view = begin_sandbox_run(tools.app_handle, tools.host_state)
            .await
            .map_err(BridgeMethodHandleError::internal_error)?;

        let runner_app = tools.app_handle.clone();

        tokio::spawn(async move {
            Box::pin(sandbox_runner(runner_app)).await;
        });

        Ok(Box::new(view))
    }
}
