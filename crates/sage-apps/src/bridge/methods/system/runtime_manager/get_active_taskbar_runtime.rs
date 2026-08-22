use async_trait::async_trait;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    SageAppRuntimeRecord, SageAppRuntimeRecordView, SystemBridgeCapability,
    find_active_taskbar_runtime, find_runtime_by_runtime_id_optional, resolve_running_app,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeManagerGetActiveTaskbarRuntime;

#[async_trait]
impl BridgeMethod for RuntimeManagerGetActiveTaskbarRuntime {
    fn name(&self) -> &'static str {
        "runtimeManager.getActiveTaskbarRuntime"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(
            SystemBridgeCapability::RuntimeManagerGetActiveTaskbarRuntime,
        )
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
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let app = resolve_running_app(tools.host_state, &ctx.app.id())
            .await
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!(
                    "failed to resolve caller runtime: {err}"
                ))
            })?;
        let host_window_label = app
            .runtime()
            .with_runtime(SageAppRuntimeRecord::host_window_label);

        let active_taskbar_runtime =
            find_active_taskbar_runtime(tools.host_state, &host_window_label).await;

        let Some(active_taskbar_runtime) = active_taskbar_runtime else {
            return Ok(Box::new(None::<SageAppRuntimeRecordView>));
        };

        let runtime: Option<SageAppRuntimeRecordView> = find_runtime_by_runtime_id_optional(
            tools.host_state,
            &active_taskbar_runtime.runtime_id(),
        )
        .await
        .map(Into::into);

        Ok(Box::new(runtime))
    }
}
