use async_trait::async_trait;

use crate::bridge::RustBridgeRequest;
use crate::bridge::methods::shared::{
    BridgeApprovalRequestResult, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError,
};
use crate::bridge::methods::{BridgeContext, BridgeMethod, BridgeTools};
use crate::capabilities::list::SystemBridgeCapability;
use crate::sandbox::runner::{begin_sandbox_run, sandbox_runner};

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
            sandbox_runner(runner_app).await;
        });

        Ok(Box::new(view))
    }
}
